# Code repurposed from `zoxide/templates/zsh.txt`.


function kn() {{
    if [[ "$#" -eq 0 ]]; then
		# no args provided

		\builtin cd
    elif [[ "$#" -eq 1 ]] && [[ "$1" = '-' ]]; then
        # only dash provided, go to previous location

        if [ -n "${{OLDPWD}}" ]; then
            \builtin cd "${{OLDPWD}}"
        fi
    else
        # otherwise, query _kn

        \builtin local __kn_result
        __kn_result="$({query_command})" \
            && \builtin cd "${{__kn_result}}"
    fi
}}
