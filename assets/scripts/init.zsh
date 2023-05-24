_update_plugins_hook() {{
	local res="$({cmd})"
	[[ -n "$res" ]] && eval "$res"
}}
trap '_update_plugins_hook' WINCH
