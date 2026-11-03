use std::cell::{Cell, RefCell};
use std::ffi::{c_void, CStr};
use std::rc::Rc;
use std::time::Instant;

use block2::RcBlock;
use nsh_platform_macos::objc2::rc::{Allocated, Retained};
use nsh_platform_macos::objc2::runtime::{AnyClass, AnyObject};
use nsh_platform_macos::objc2::{define_class, msg_send, DefinedClass, MainThreadOnly};
use nsh_platform_macos::objc2_foundation::{NSObjectProtocol, NSRect, NSSize, NSString};
use objc2_foundation::NSData;
use spike_epub::{normalize_href, pack, PackedBook};
use spike_report::Anchor;

const PAGE_JS: &str = r#"
(function () {
  const style = document.createElement('style');
  style.textContent = `
    html, body { height: 100% !important; margin: 0 !important; background: #f7f3ea !important; color: #1c1915 !important; }
    body {
      column-width: 100vw !important;
      column-gap: 0 !important;
      column-fill: auto !important;
      height: 100vh !important;
      overflow-x: auto !important;
      overflow-y: hidden !important;
    }
    mark.spike { background: rgba(255, 214, 64, 0.55); color: inherit; }
  `;
  document.documentElement.appendChild(style);
  const root = document.body;
  function box() {
    const w = Math.max(1, window.innerWidth);
    const pages = Math.max(1, Math.round(root.scrollWidth / w));
    const page = Math.min(pages, Math.round(root.scrollLeft / w) + 1);
    return { page: page, pages: pages, left: pages - page };
  }
  window.spikeBox = box;
  function report() {
    window.webkit.messageHandlers.spike.postMessage(JSON.stringify(box()));
  }
  window.spikeTurn = function (delta) {
    const t = performance.now();
    const w = Math.max(1, window.innerWidth);
    root.scrollLeft = Math.max(0, root.scrollLeft + delta * w);
    const b = box();
    b.ms = performance.now() - t;
    report();
    return JSON.stringify(b);
  };
  requestAnimationFrame(report);
  function offsets(range) {
    const pre = range.cloneRange();
    pre.selectNodeContents(root);
    pre.setEnd(range.startContainer, range.startOffset);
    const start = pre.toString().length;
    const text = range.toString();
    return { start: start, end: start + text.length, text: text };
  }
  window.spikePaint = function (start, end) {
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
    let seen = 0;
    let sNode = null, sOff = 0, eNode = null, eOff = 0;
    while (walker.nextNode()) {
      const node = walker.currentNode;
      const next = seen + node.textContent.length;
      if (!sNode && next > start) { sNode = node; sOff = start - seen; }
      if (!eNode && next >= end) { eNode = node; eOff = end - seen; break; }
      seen = next;
    }
    if (!sNode || !eNode) return 'miss';
    document.querySelectorAll('mark.spike').forEach((m) => {
      const parent = m.parentNode;
      while (m.firstChild) parent.insertBefore(m.firstChild, m);
      parent.removeChild(m);
    });
    const range = document.createRange();
    range.setStart(sNode, sOff);
    range.setEnd(eNode, eOff);
    const mark = document.createElement('mark');
    mark.className = 'spike';
    try { range.surroundContents(mark); } catch (e) { return 'split'; }
    return 'ok';
  };
  document.addEventListener('mouseup', function () {
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0 || sel.isCollapsed) return;
    const hit = offsets(sel.getRangeAt(0));
    if (!hit.text.trim()) return;
    window.webkit.messageHandlers.spike.postMessage(JSON.stringify(hit));
  });
})();
"#;

extern "C" {
    fn dlopen(filename: *const i8, flags: i32) -> *mut c_void;
}

fn ensure_webkit() {
    unsafe {
        dlopen(
            c"/System/Library/Frameworks/WebKit.framework/WebKit".as_ptr(),
            1,
        );
    }
}

fn class(name: &CStr) -> Option<&'static AnyClass> {
    AnyClass::get(name)
}

fn poke(handler: &WebHandler) {
    let host = handler.ivars().host.get();
    if host.is_null() {
        return;
    }
    unsafe {
        let _: () = msg_send![host, setNeedsDisplay: true];
    }
}

struct Files {
    book: PackedBook,
}

struct HandlerIvars {
    files: Rc<Files>,
    inbox: Rc<RefCell<Inbox>>,
    href: String,
    opened: Instant,
    host: std::cell::Cell<*mut AnyObject>,
    choice: Rc<Cell<crate::reading::Choice>>,
}

#[derive(Default)]
pub struct Inbox {
    pub pages: Option<(usize, usize)>,
    pub open_ms: Option<f64>,
    pub turn_ms: Vec<f64>,
    pub frame_ms: Vec<f64>,
    pub scripted_turn: bool,
    pub anchor: Option<Anchor>,
    pub ready: bool,
    pub reload: bool,
    pub did_reload: bool,
    pub turn_started: Option<Instant>,
}

define_class!(
    #[unsafe(super(objc2::runtime::NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = HandlerIvars]
    struct WebHandler;

    unsafe impl NSObjectProtocol for WebHandler {}

    impl WebHandler {
        #[unsafe(method(webView:startURLSchemeTask:))]
        fn start_task(&self, _web: &AnyObject, task: &AnyObject) {
            let req: Retained<AnyObject> = unsafe { msg_send![task, request] };
            let url: Retained<objc2_foundation::NSURL> = unsafe { msg_send![&req, URL] };
            let path = url.path().map(|s| s.to_string()).unwrap_or_default();
            let key = normalize_href(&path);
            let mime = mime_of(&key);
            let bytes = self.ivars().files.book.files.get(&key).cloned();
            unsafe {
                let Some(bytes) = bytes else {
                    let err: Retained<AnyObject> = msg_send![
                        class(c"NSError").unwrap(),
                        errorWithDomain: &*NSString::from_str("spike"),
                        code: 404isize,
                        userInfo: std::ptr::null::<AnyObject>()
                    ];
                    let _: () = msg_send![task, didFailWithError: &*err];
                    return;
                };
                let body = if key.ends_with(".xhtml") || key.ends_with(".html") || key.ends_with(".htm") {
                    inject(&bytes, self.ivars().choice.get())
                } else {
                    bytes
                };
                let resp_alloc: Allocated<AnyObject> = msg_send![class(c"NSURLResponse").unwrap(), alloc];
                let resp: Retained<AnyObject> = msg_send![
                    resp_alloc,
                    initWithURL: &*url,
                    MIMEType: &*NSString::from_str(mime),
                    expectedContentLength: body.len() as isize,
                    textEncodingName: &*NSString::from_str("utf-8")
                ];
                let data = NSData::with_bytes(&body);
                let _: () = msg_send![task, didReceiveResponse: &*resp];
                let _: () = msg_send![task, didReceiveData: &*data];
                let _: () = msg_send![task, didFinish];
            }
        }

        #[unsafe(method(webView:stopURLSchemeTask:))]
        fn stop_task(&self, _web: &AnyObject, _task: &AnyObject) {}

        #[unsafe(method(userContentController:didReceiveScriptMessage:))]
        fn message(&self, controller: &AnyObject, message: &AnyObject) {
            let _ = controller;
            let body: Retained<NSString> = unsafe { msg_send![message, body] };
            let text = body.to_string();
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                if let (Some(page), Some(pages)) = (value["page"].as_u64(), value["pages"].as_u64())
                {
                    let mut inbox = self.ivars().inbox.borrow_mut();
                    let previous = inbox.pages;
                    inbox.pages = Some((page as usize, pages as usize));
                    if let Some(frames) = value.get("frames").and_then(|item| item.as_array()) {
                        inbox.frame_ms.extend(frames.iter().filter_map(|item| item.as_f64()));
                    }
                    if previous.is_some() && previous != inbox.pages {
                        if let Some(started) = inbox.turn_started.take() {
                            inbox.turn_ms.push(started.elapsed().as_secs_f64() * 1000.0);
                        }
                    }
                    drop(inbox);
                    write_inbox(&self.ivars().inbox.borrow(), &self.ivars().href);
                    crate::web::poke(self);
                    return;
                }
                let start = value["start"].as_u64().unwrap_or(0) as u32;
                let end = value["end"].as_u64().unwrap_or(0) as u32;
                let selected = value["text"].as_str().unwrap_or("").to_string();
                if !selected.trim().is_empty() {
                    let mut inbox = self.ivars().inbox.borrow_mut();
                    inbox.anchor = Some(Anchor::Epub {
                        spine_href: self.ivars().href.clone(),
                        start,
                        end,
                        text: selected,
                    });
                    if std::env::var("SPIKE_RELOAD").as_deref() == Ok("1") && !inbox.did_reload {
                        inbox.reload = true;
                    }
                    drop(inbox);
                    write_inbox(&self.ivars().inbox.borrow(), &self.ivars().href);
                }
            }
        }

        #[unsafe(method(webView:didFinishNavigation:))]
        fn finished(&self, web: &AnyObject, _nav: &AnyObject) {
            let open_ms = self.ivars().opened.elapsed().as_secs_f64() * 1000.0;
            {
                let mut inbox = self.ivars().inbox.borrow_mut();
                if inbox.open_ms.is_none() {
                    inbox.open_ms = Some(open_ms);
                }
                inbox.ready = true;
            }
            unsafe {
                let superview: *mut AnyObject = msg_send![web, superview];
                if !superview.is_null() {
                    let _: () = msg_send![superview, setNeedsDisplay: true];
                }
            }
            eval(
                web,
                "JSON.stringify(window.spikeBox())",
                self.ivars().inbox.clone(),
                self.ivars().href.clone(),
                false,
            );
            if self.ivars().inbox.borrow().anchor.is_none()
                && std::env::var("SPIKE_SELECT").as_deref() == Ok("1")
            {
                let needle = std::env::var("SPIKE_NEEDLE").unwrap_or_else(|_| {
                    if self.ivars().href.contains("25271") || self.ivars().href.contains("OEBPS") {
                        "朝花夕拾".into()
                    } else {
                        "truth universally acknowledged".into()
                    }
                });
                let js = format!(
                    r#"(function() {{
                      const root = document.body;
                      const text = root.textContent || '';
                      const needle = {needle:?};
                      const at = text.indexOf(needle);
                      if (at < 0) return 'miss';
                      window.spikePaint(at, at + needle.length);
                      const hit = {{start: at, end: at + needle.length, text: needle}};
                      window.webkit.messageHandlers.spike.postMessage(JSON.stringify(hit));
                      return JSON.stringify(hit);
                    }})()"#
                );
                eval(
                    web,
                    &js,
                    self.ivars().inbox.clone(),
                    self.ivars().href.clone(),
                    false,
                );
            }
            let anchor = self.ivars().inbox.borrow().anchor.clone();
            if let Some(Anchor::Epub { start, end, .. }) = anchor {
                eval(
                    web,
                    &format!("window.spikePaint({start}, {end})"),
                    self.ivars().inbox.clone(),
                    self.ivars().href.clone(),
                    false,
                );
            }
        }
    }
);

pub(crate) fn write_inbox(inbox: &Inbox, href: &str) {
    if inbox.open_ms.is_none() && inbox.pages.is_none() && inbox.anchor.is_none() {
        return;
    }
    let _ = std::fs::create_dir_all("/tmp/superreader-spike");
    let body = serde_json::json!({
        "href": href,
        "open_ms": inbox.open_ms,
        "turn_ms": inbox.turn_ms,
        "frame_ms": inbox.frame_ms,
        "frame_floor_ok": crate::reading::frames_within_floor(&inbox.frame_ms),
        "pages": inbox.pages,
        "anchor": inbox.anchor,
    });
    let _ = std::fs::write(
        "/tmp/superreader-spike/webkit.json",
        serde_json::to_string_pretty(&body).unwrap_or_default(),
    );
}

fn eval(web: &AnyObject, js: &str, inbox: Rc<RefCell<Inbox>>, href: String, turn: bool) {
    let js = NSString::from_str(js);
    let block = RcBlock::new(move |value: *mut AnyObject, _error: *mut AnyObject| {
        if value.is_null() {
            return;
        }
        let text = unsafe {
            let desc: Retained<NSString> = msg_send![value, description];
            desc.to_string()
        };
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) else {
            return;
        };
        let mut inbox = inbox.borrow_mut();
        if let (Some(page), Some(pages)) = (parsed["page"].as_u64(), parsed["pages"].as_u64()) {
            inbox.pages = Some((page as usize, pages as usize));
        }
        if turn {
            if let Some(ms) = parsed["ms"].as_f64() {
                inbox.turn_ms.push(ms);
            }
        }
        if let Some(text) = parsed["text"].as_str() {
            if !text.trim().is_empty() && inbox.anchor.is_none() {
                inbox.anchor = Some(Anchor::Epub {
                    spine_href: href.clone(),
                    start: parsed["start"].as_u64().unwrap_or(0) as u32,
                    end: parsed["end"].as_u64().unwrap_or(0) as u32,
                    text: text.to_string(),
                });
                if std::env::var("SPIKE_RELOAD").as_deref() == Ok("1") && !inbox.did_reload {
                    inbox.reload = true;
                }
                write_inbox(&inbox, &href);
            }
        }
    });
    unsafe {
        let _: () = msg_send![web, evaluateJavaScript: &*js, completionHandler: &*block];
    }
}

fn inject(bytes: &[u8], choice: crate::reading::Choice) -> Vec<u8> {
    let mut html = String::from_utf8_lossy(bytes).into_owned();
    let mut script = format!("<script>{PAGE_JS}</script>");
    if let Some(view) = crate::reading::view(choice) {
        script.push_str("<script>");
        script.push_str(&crate::reading::page_script(view));
        script.push_str("</script>");
    }
    if let Some(at) = html.to_ascii_lowercase().rfind("</body>") {
        html.insert_str(at, &script);
    } else {
        html.push_str(&script);
    }
    html.into_bytes()
}

fn mime_of(path: &str) -> &'static str {
    if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".xhtml") || path.ends_with(".html") || path.ends_with(".htm") {
        "text/html"
    } else {
        "application/octet-stream"
    }
}

pub struct WebChapter {
    web: Retained<AnyObject>,
    _handler: Retained<WebHandler>,
    pub inbox: Rc<RefCell<Inbox>>,
    href: String,
}

impl WebChapter {
    pub fn open(
        parent: &AnyObject,
        path: &str,
        choice: Rc<Cell<crate::reading::Choice>>,
    ) -> Result<Self, String> {
        ensure_webkit();
        let packed = pack(path)?;
        let href = packed.href.clone();
        let inbox = Rc::new(RefCell::new(Inbox::default()));
        let mtm = nsh_platform_macos::objc2::MainThreadMarker::new().expect("main");
        let handler = mtm
            .alloc::<WebHandler>()
            .set_ivars(HandlerIvars {
                files: Rc::new(Files { book: packed }),
                inbox: inbox.clone(),
                href: href.clone(),
                opened: Instant::now(),
                host: std::cell::Cell::new(std::ptr::null_mut()),
                choice,
            });
        let handler: Retained<WebHandler> = unsafe { msg_send![super(handler), init] };
        handler.ivars().host.set(parent as *const AnyObject as *mut AnyObject);
        let config: Retained<AnyObject> = unsafe { msg_send![class(c"WKWebViewConfiguration").unwrap(), new] };
        unsafe {
            let _: () = msg_send![&config, setURLSchemeHandler: &*handler, forURLScheme: &*NSString::from_str("epub")];
            let content: Retained<AnyObject> = msg_send![&config, userContentController];
            let _: () = msg_send![&content, addScriptMessageHandler: &*handler, name: &*NSString::from_str("spike")];
        }
        let alloc: Allocated<AnyObject> = unsafe { msg_send![class(c"WKWebView").unwrap(), alloc] };
        let web: Retained<AnyObject> = unsafe {
            msg_send![alloc, initWithFrame: NSRect::ZERO, configuration: &*config]
        };
        unsafe {
            let _: () = msg_send![&web, setNavigationDelegate: &*handler];
            let _: () = msg_send![parent, addSubview: &*web];
            let url = NSString::from_str(&format!("epub://book{href}"));
            let nsurl = objc2_foundation::NSURL::URLWithString(&url).ok_or("url")?;
            let request: Retained<AnyObject> = msg_send![class(c"NSURLRequest").unwrap(), requestWithURL: &*nsurl];
            let _: Option<Retained<AnyObject>> = msg_send![&web, loadRequest: &*request];
        }
        Ok(WebChapter {
            web,
            _handler: handler,
            inbox,
            href,
        })
    }

    pub fn place(&self, width: f64, height: f64) {
        unsafe {
            let _: () = msg_send![
                &*self.web,
                setFrame: NSRect::new(
                    objc2_foundation::NSPoint::new(0.0, 0.0),
                    NSSize::new(width, height)
                )
            ];
        }
    }

    pub fn turn(&self, delta: i32) {
        self.inbox.borrow_mut().turn_started = Some(Instant::now());
        let inbox = self.inbox.clone();
        eval(
            &self.web,
            &format!("window.spikeTurn({delta})"),
            inbox,
            self.href.clone(),
            true,
        );
    }

    pub fn href(&self) -> &str {
        &self.href
    }

    pub fn reload(&self) {
        unsafe {
            let _: Option<Retained<AnyObject>> = msg_send![&*self.web, reload];
        }
    }

    pub fn repaint(&self, start: u32, end: u32) {
        eval(
            &self.web,
            &format!("window.spikePaint({start}, {end})"),
            self.inbox.clone(),
            self.href.clone(),
            false,
        );
    }
}
