# Code repurposed from `zoxide/templates/bash.txt`.


function kn() {{
    if [[ "$#" -eq 0 ]]; then
        \builtin exit 0 # no args provided, do nothing
    elif [[ "$#" -eq 1 ]] && [[ "$1" = '-' ]]; then
        # only dash provided, go to previous location if it exists

        if [[ -n "${{OLDPWD}}" ]]; then
            \builtin cd "${{OLDPWD}}"
        fi
    else
        # otherwise, query _kn

        \builtin local __kn_result
        __kn_result="$({query_command})" && \builtin cd "${{__kn_result}}"
    fi
}}
