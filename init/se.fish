# code repurposed from zoxide/templates/fish.txt

function se
    set argc (count $argv)

    echo "running"

    if test $argc -eq 0
        # no args provided, go to home dir
        
        cd $HOME
    else if begin; test $argc -eq 1; and test $argv[1] = '-'; end
        # only dash provided, go to previous location
        
        cd -
    else if begin; test $argc -ge 1; and test -d $argv[1]; end
        # at least 1 arg provided and the first one is a dir

        echo "with starting dir"

        if test $argc -eq 1
            echo "no slices"
            set -l __se_result (command _se query --start-dir $argv[1])

            and if test -d $__se_result
                echo "$__se_result"
                cd $__se_result
            end
        else
            echo "with slices"
            set -l __se_result (command _se query --start-dir $argv[1] $argv[2..-1])
            echo "$__se_result"

            and if test -d $__se_result
                cd $__se_result
            end
        end
    else
        echo "just slices"
        # at least 1 arg provided and the first one is not a dir
        
        set -l __se_result (command _se query $argv)
        echo "$__se_result"

        and if test -d $__se_result
            cd $__se_result
        end
    end
end
