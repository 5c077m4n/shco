_update_plugins_hook() {{
	local res="$({location})"
	[[ -n "$res" ]] && eval "$res"
}}
autoload -Uz add-zsh-hook
add-zsh-hook preexec _update_plugins_hook
