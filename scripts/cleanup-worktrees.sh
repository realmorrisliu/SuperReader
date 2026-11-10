#!/bin/sh
# Remove stale SuperReader worktrees whose PR is merged or closed.
# Dry-run unless --apply. Idempotent. Never uses --force.
set -eu

apply=0
case "${1:-}" in
  "") ;;
  --dry-run) ;;
  --apply) apply=1 ;;
  *)
    echo "usage: scripts/cleanup-worktrees.sh [--dry-run|--apply]" >&2
    exit 2
    ;;
esac

cd "$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"

url=$(git remote get-url origin)
case "$url" in
  *realmorrisliu/SuperReader*) ;;
  *)
    echo "origin is not realmorrisliu/SuperReader ($url). Nothing removed." >&2
    exit 1
    ;;
esac

if ! command -v gh >/dev/null 2>&1; then
  echo "gh is not on PATH. Nothing removed." >&2
  exit 1
fi

current=$(git rev-parse --show-toplevel)

keep_named() {
  case "$1" in
    cursor/apple-books-chrome-a42e|cursor/epub-native-spike-3b1f)
      return 0
      ;;
  esac
  case "$1" in
    *typesetter*|*engine*) return 0 ;;
  esac
  return 1
}

# A process whose cwd is this worktree or a directory inside it.
busy() {
  lsof -a -d cwd -Fn 2>/dev/null | awk -v p="$1" '
    /^n/ {
      cwd = substr($0, 2)
      if (cwd == p || index(cwd, p "/") == 1) found = 1
    }
    END { exit found ? 0 : 1 }
  '
}

pr_states() {
  gh pr list --repo realmorrisliu/SuperReader --state all --head "$1" --json state --jq '.[].state'
}

dirty() {
  [ -n "$(git -C "$1" status --porcelain)" ]
}

remove_build_dirs() {
  path=$1
  find "$path" -name .git -prune -o -type d \( -name target -o -name .nsh \) -print | while IFS= read -r dir; do
    case "$dir" in
      "$path"/*) ;;
      *) continue ;;
    esac
    base=$(basename "$dir")
    if [ "$base" = "target" ]; then
      if [ ! -e "$dir/CACHEDIR.TAG" ] && [ ! -d "$dir/debug" ] && [ ! -d "$dir/release" ]; then
        continue
      fi
    fi
    if [ "$apply" -eq 1 ]; then
      rm -rf "$dir"
      echo "deleted build dir $dir"
    else
      echo "would delete build dir $dir"
    fi
  done
}

plan=/tmp/superreader-worktree-plan.$$
trap 'rm -f "$plan"' EXIT
git worktree list --porcelain > "$plan"

path=
branch=
flush() {
  [ -n "${path:-}" ] || return 0
  short=${branch#refs/heads/}
  reason=""
  action=keep
  if [ -d "$path/.git" ]; then
    reason="main clone"
  elif [ "$path" = "$current" ]; then
    reason="current worktree"
  elif keep_named "$short"; then
    reason="named keep ($short)"
  elif [ -z "$short" ]; then
    reason="detached HEAD"
  elif dirty "$path"; then
    reason="uncommitted changes"
  elif busy "$path"; then
    reason="live process"
  else
    states=$(pr_states "$short" || true)
    if [ -z "$states" ]; then
      reason="no pull request"
    else
      open=0
      closed=0
      for state in $states; do
        case "$state" in
          OPEN) open=1 ;;
          MERGED|CLOSED) closed=1 ;;
          *) open=1 ;;
        esac
      done
      if [ "$open" -eq 1 ]; then
        reason="open pull request"
      elif [ "$closed" -eq 1 ]; then
        action=remove
        reason="pull request ${states}"
      else
        reason="pull request state unknown"
      fi
    fi
  fi
  size=$(du -sh -I .worktrees "$path" 2>/dev/null | awk '{print $1}')
  if [ "$action" = "remove" ]; then
    echo "REMOVE $path branch=$short pr=${states} size=$size"
    remove_build_dirs "$path"
    if [ "$apply" -eq 1 ]; then
      if git worktree remove "$path"; then
        echo "removed worktree $path"
        if git branch -D "$short"; then
          echo "deleted branch $short"
        else
          echo "SKIP branch $short: git branch -D failed"
        fi
      else
        echo "SKIP $path: git worktree remove refused"
      fi
    fi
  else
    echo "KEEP $path branch=${short:-detached} size=$size ($reason)"
  fi
  path=
  branch=
}

while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in
    "") flush ;;
    worktree\ *) path=${line#worktree } ;;
    branch\ *) branch=${line#branch } ;;
  esac
done < "$plan"
flush

if [ "$apply" -eq 1 ]; then
  git worktree prune
  echo "pruned"
else
  echo "dry-run. Pass --apply to remove."
fi
