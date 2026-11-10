const fs = require("fs");
const source = fs.readFileSync(process.argv[2], "utf8");
const end = source.indexOf("(function () {");
if (end < 0) throw new Error("missing justify wrapper");
eval(source.slice(0, end));

function text(value) {
  return { nodeType: 3, textContent: value, childNodes: [] };
}

function elem(tag, attrs, children) {
  return {
    nodeType: 1,
    tagName: tag,
    attrs: attrs,
    childNodes: children,
    hasAttribute(name) {
      return Object.prototype.hasOwnProperty.call(this.attrs, name);
    },
  };
}

function wire(root) {
  function walk(el) {
    let prev = null;
    for (const node of el.childNodes) {
      node.parentNode = el;
      if (node.nodeType === 1) {
        node.previousElementSibling = prev;
        prev = node;
        walk(node);
      }
    }
  }
  walk(root);
}

function assert(cond, message) {
  if (!cond) throw new Error(message);
}

const wrapped = elem("P", {}, [
  elem("SPAN", { "data-kp-line": "" }, [
    text("acknowl"),
    elem("SPAN", { "data-kp-hyphen": "" }, []),
  ]),
  elem("BR", { "data-kp": "" }, []),
  elem("SPAN", { "data-kp-line": "" }, [text("edged")]),
]);
wire(wrapped);
assert(JSON.stringify(harvestCuts(wrapped)) === JSON.stringify([[7, "h"]]), "wrapped hyphen");

const glue = elem("P", {}, [
  elem("SPAN", { "data-kp-line": "" }, [text("hello")]),
  elem("BR", { "data-kp": "" }, []),
  elem("SPAN", { "data-kp-line": "" }, [text("there")]),
]);
wire(glue);
assert(JSON.stringify(harvestCuts(glue)) === JSON.stringify([[5, "g"]]), "wrapped glue");

const bare = elem("P", {}, [
  text("ac"),
  elem("SPAN", { "data-kp-hyphen": "" }, []),
  elem("BR", { "data-kp": "" }, []),
  text("know"),
]);
wire(bare);
assert(JSON.stringify(harvestCuts(bare)) === JSON.stringify([[2, "h"]]), "bare hyphen");

console.log("harvest cuts ok");
