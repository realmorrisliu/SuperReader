#!/usr/bin/env bash
# Regenerates assets/icons/*.svg from assets/icons/manifest.toml.
# Talks to IconMCP with curl. Calls initialize, then get_icon only.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
manifest="$root/assets/icons/manifest.toml"
url="https://iconmcp.com/mcp"
ua="superreader-icons/0.1"

if [[ ! -f "$manifest" ]]; then
  echo "missing $manifest" >&2
  exit 1
fi

stroke=1.25
radius=2
size=16
color="currentColor"
rows=""

while IFS='|' read -r kind a b c d e f; do
  if [[ "$kind" == "GLOBAL" ]]; then
    stroke="$a"
    radius="$b"
    size="$c"
    color="$d"
  elif [[ "$kind" == "ICON" ]]; then
    rows+="$a|$b|$c|$d|$e|$f"$'\n'
  fi
done < <(
  awk '
    function unquote(s) {
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", s)
      if (s ~ /^".*"$/) {
        sub(/^"/, "", s)
        sub(/"$/, "", s)
      }
      return s
    }
    function flush() {
      if (name != "") {
        print "ICON|" name "|" icon "|" setname "|" license "|" style "|" query
      }
    }
    BEGIN { stroke = "1.25"; radius = "2"; size = "16"; color = "currentColor" }
    /^[[:space:]]*#/ || /^[[:space:]]*$/ { next }
    /^\[\[icon\]\]/ {
      flush()
      name = icon = setname = license = style = query = ""
      in_icon = 1
      next
    }
    {
      eq = index($0, "=")
      if (eq == 0) next
      key = substr($0, 1, eq - 1)
      val = unquote(substr($0, eq + 1))
      gsub(/[[:space:]]/, "", key)
      if (!in_icon) {
        if (key == "stroke") stroke = val
        else if (key == "radius") radius = val
        else if (key == "size") size = val
        else if (key == "color") color = val
        next
      }
      if (key == "name") name = val
      else if (key == "icon") icon = val
      else if (key == "set") setname = val
      else if (key == "license") license = val
      else if (key == "style") style = val
      else if (key == "query") query = val
    }
    END {
      flush()
      print "GLOBAL|" stroke "|" radius "|" size "|" color
    }
  ' "$manifest"
)

if [[ -z "$rows" ]]; then
  echo "manifest has no icons" >&2
  exit 1
fi

hdr="$(mktemp)"
body="$(mktemp)"
trap 'rm -f "$hdr" "$body"' EXIT

curl -fsS --max-time 40 -A "$ua" -D "$hdr" -o "$body" \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'mcp-protocol-version: 2024-11-05' \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"superreader-icons","version":"0.1.0"}}}' \
  "$url"

session="$(awk 'tolower($1) == "mcp-session-id:" { print $2 }' "$hdr" | tr -d '\r')"
session_header=()
if [[ -n "$session" ]]; then
  session_header=(-H "mcp-session-id: $session")
fi

curl -fsS --max-time 40 -A "$ua" -o /dev/null \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'mcp-protocol-version: 2024-11-05' \
  ${session_header[@]+"${session_header[@]}"} \
  -d '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  "$url"

id=2
while IFS='|' read -r name icon setname license style query; do
  [[ -z "${name:-}" ]] && continue
  if [[ -z "$icon" ]]; then
    icon="$name"
  fi
  if [[ "$style" != "outline" && "$style" != "solid" ]]; then
    echo "$name: style must be outline or solid" >&2
    exit 1
  fi
  if [[ -z "$setname" || -z "$license" || -z "$query" ]]; then
    echo "$name: set, license, and query are required" >&2
    exit 1
  fi
  payload="$(jq -nc \
    --argjson id "$id" \
    --arg icon "$icon" \
    --arg style "$style" \
    --arg color "$color" \
    --argjson size "$size" \
    --argjson stroke "$stroke" \
    --argjson radius "$radius" \
    '{jsonrpc:"2.0",id:$id,method:"tools/call",params:{name:"get_icon",arguments:{name:$icon,format:"svg",style:$style,size:$size,stroke:$stroke,radius:$radius,color:$color}}}')"
  curl -fsS --max-time 40 -A "$ua" -o "$body" \
    -H 'Content-Type: application/json' \
    -H 'Accept: application/json, text/event-stream' \
    -H 'mcp-protocol-version: 2024-11-05' \
    ${session_header[@]+"${session_header[@]}"} \
    -d "$payload" \
    "$url"
  svg="$(jq -er '.result.content[0].text' "$body")"
  if [[ "$svg" != "<svg"* ]]; then
    echo "$name: get_icon did not return an svg" >&2
    exit 1
  fi
  dest="$root/assets/icons/$name.svg"
  tmp="$dest.tmp"
  printf '%s\n' "$svg" > "$tmp"
  if [[ -f "$dest" ]] && cmp -s "$tmp" "$dest"; then
    rm -f "$tmp"
  else
    mv "$tmp" "$dest"
  fi
  echo "wrote $name.svg ($setname, $license, $style)"
  id=$((id + 1))
done <<< "$rows"
