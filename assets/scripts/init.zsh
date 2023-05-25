__shco_source_plugins() {{
	local res="$({cmd_source})"
	[[ -n "$res" ]] && eval "$res"
}}
trap '__shco_source_plugins' WINCH

__shco_sync_plugins() {{
	local res="$({cmd_sync})"
	[[ -n "$res" ]] && eval "$res"
}}
autoload -Uz add-zsh-hook
add-zsh-hook precmd __shco_sync_plugins
