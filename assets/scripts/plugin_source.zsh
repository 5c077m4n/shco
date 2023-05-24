local -a initfiles=(
	{plug_dir}/{author}/{plug_name}/{plug_name}.{{plugin.,}}{{z,}}sh{{-theme,}}(N)
	{plug_dir}/{author}/{plug_name}/*.{{plugin.,}}{{z,}}sh{{-theme,}}(N)
)
(( $#initfiles )) && source $initfiles[1]
