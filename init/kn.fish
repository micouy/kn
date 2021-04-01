# code repurposed from zoxide/templates/fish.txt

function kn
    set argc (count $argv)

    if test $argc -eq 0
        # no args provided, go to home dir
        
        cd $HOME
    else if begin; test $argc -eq 1; and test $argv[1] = '-'; end
        # only dash provided, go to previous location
        
        cd -
    else if begin; test $argc -ge 1; and test -d $argv[1]; end
        # at least 1 arg provided and the first one is a dir

        if test $argc -eq 1
            set -l __kn_result (command _kn query --start-dir $argv[1])

            and if test -d $__kn_result
                cd $__kn_result
            end
        else
            set -l __kn_result (command _kn query --start-dir $argv[1] $argv[2..-1])

            and if test -d $__kn_result
                cd $__kn_result
            end
        end
    else
        # at least 1 arg provided and the first one is not a dir
        
        set -l __kn_result (command _kn query $argv)

        and if test -d $__kn_result
            cd $__kn_result
        end
    end
end
