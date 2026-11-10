#!/usr/bin/env bash
# Machine-wide serialize wrapper for scripts/verify (see docs/tooling.md).
set -eu -o pipefail

LOCK_DIR=/tmp/superreader-verify.lock
MAX_LOAD="${SR_VERIFY_MAX_LOAD:-8}"
LOCK_DIR_MIN_AGE_SEC=60
VERIFY_CHILD_PID=

# Non-interactive stdin makes `read -t` return immediately; a fifo keeps `read` blocking.
_verify_pause_fifo=$(mktemp -u "${LOCK_DIR}.pause.XXXXXX")
mkfifo "$_verify_pause_fifo"
exec 3<>"$_verify_pause_fifo"
rm -f "$_verify_pause_fifo"

pause() {
  local i
  for ((i = 0; i < $1; i++)); do
    read -r -t 1 _ <&3 || :
  done
}

close_pause_fifo() {
  exec 3>&- 2>/dev/null || true
}

one_min_load() {
  if sysctl -n vm.loadavg &>/dev/null; then
    sysctl -n vm.loadavg | tr -d '{}' | awk '{print $1}'
  else
    awk '{print $1}' /proc/loadavg
  fi
}

lock_dir_age_sec() {
  local now mtime
  if stat -c %Y "$LOCK_DIR" &>/dev/null; then
    mtime=$(stat -c %Y "$LOCK_DIR")
    now=$(date +%s)
  else
    mtime=$(stat -f %m "$LOCK_DIR")
    now=$(date +%s)
  fi
  echo $((now - mtime))
}

lock_holder() {
  local pid branch started
  pid=$(cat "$LOCK_DIR/pid" 2>/dev/null || echo '?')
  branch=$(cat "$LOCK_DIR/branch" 2>/dev/null || echo '?')
  started=$(cat "$LOCK_DIR/started" 2>/dev/null || echo '?')
  echo "pid ${pid} branch ${branch} since ${started}"
}

clear_stale_lock() {
  [[ -d "$LOCK_DIR" ]] || return 1

  if [[ ! -f "$LOCK_DIR/pid" ]]; then
    if [[ "$(lock_dir_age_sec)" -lt "$LOCK_DIR_MIN_AGE_SEC" ]]; then
      return 1
    fi
    echo "clearing verify lock without pid metadata (older than ${LOCK_DIR_MIN_AGE_SEC}s)"
    rm -rf "$LOCK_DIR"
    return 0
  fi

  local holder
  holder=$(cat "$LOCK_DIR/pid")
  if kill -0 "$holder" 2>/dev/null; then
    return 1
  fi
  echo "clearing stale verify lock (pid ${holder})"
  rm -rf "$LOCK_DIR"
  return 0
}

release_lock() {
  [[ -d "$LOCK_DIR" ]] || return 0
  [[ -f "$LOCK_DIR/pid" ]] || return 0
  [[ "$(cat "$LOCK_DIR/pid")" == "$$" ]] || return 0
  rm -rf "$LOCK_DIR"
}

on_shell_exit() {
  close_pause_fifo
  release_lock
}

on_signal() {
  local sig=$1 code=$2 child
  trap - EXIT INT TERM
  child=$VERIFY_CHILD_PID
  if [[ -n "$child" ]] && kill -0 "$child" 2>/dev/null; then
    kill -s "$sig" "$child" 2>/dev/null || true
    set +e
    wait "$child" 2>/dev/null
    set -e
  fi
  close_pause_fifo
  release_lock
  exit "$code"
}

trap on_shell_exit EXIT
trap 'on_signal INT 130' INT
trap 'on_signal TERM 143' TERM

acquire_lock() {
  while true; do
    if clear_stale_lock; then
      continue
    fi
    if mkdir "$LOCK_DIR" 2>/dev/null; then
      local branch
      branch=$(git branch --show-current 2>/dev/null || echo unknown)
      printf '%s\n' "$$" >"$LOCK_DIR/pid"
      printf '%s\n' "$branch" >"$LOCK_DIR/branch"
      date -u +%Y-%m-%dT%H:%M:%SZ >"$LOCK_DIR/started"
      break
    fi
    echo "waiting for verify lock (held by $(lock_holder))"
    pause 30
  done
}

wait_for_load() {
  local load
  while true; do
    load=$(one_min_load)
    if awk -v l="$load" -v m="$MAX_LOAD" 'BEGIN { exit (l+0 < m+0) ? 0 : 1 }'; then
      break
    fi
    echo "waiting for load < ${MAX_LOAD} (now ${load})"
    pause 10
  done
}

acquire_lock
wait_for_load

load=$(one_min_load)
echo "verify starting: 1m loadavg=${load} (max ${MAX_LOAD})"

status=0
if [[ "${SR_VERIFY_STUB:-}" == "1" ]]; then
  (
    echo "verify stub: scripts/verify $*"
    echo "verify stub start $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    pause "${SR_VERIFY_STUB_SLEEP:-2}"
    echo "verify stub end $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    exit "${SR_VERIFY_STUB_EXIT:-0}"
  ) &
  VERIFY_CHILD_PID=$!
  wait "$VERIFY_CHILD_PID" || status=$?
else
  scripts/verify "$@" &
  VERIFY_CHILD_PID=$!
  wait "$VERIFY_CHILD_PID" || status=$?
fi

exit "$status"
