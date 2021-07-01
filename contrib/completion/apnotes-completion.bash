_apnotes_get_notes() {
    # https://stackoverflow.com/questions/1146098/properly-handling-spaces-and-quotes-in-bash-completion
    # Get the currently completing word
    local CWORD=${COMP_WORDS[COMP_CWORD]}

    # This is our word list (in a bash array for convenience)
    if [ -n "$1" ] && [ "$1" = "undelete" ]; then
        local WORD_LIST=$(apnotes list --names --deleted 2>&1)
    else
        local WORD_LIST=$(apnotes list --names 2>&1) 
    fi
    # Commands below depend on this IFS
    local IFS=$'\n'

    # Filter our candidates
    CANDIDATES=($(compgen -W "${WORD_LIST[*]}" -- "$CWORD"))

    # Correctly set our candidates to COMPREPLY
    if [ ${#CANDIDATES[*]} -eq 0 ]; then
        COMPREPLY=()
    else
        COMPREPLY=($(printf '%q\n' "${CANDIDATES[@]}"))
    fi

    return 0
}

_apnotes() {
    local i cur prev opts cmd
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${i}" in
            apnotes)
                cmd="apnotes"
                ;;
            
            backup)
                cmd+="__backup"
                ;;
            delete)
                cmd+="__delete"
                ;;
            edit)
                cmd+="__edit"
                ;;
            help)
                cmd+="__help"
                ;;
            list)
                cmd+="__list"
                ;;
            merge)
                cmd+="__merge"
                ;;
            new)
                cmd+="__new"
                ;;
            print)
                cmd+="__print"
                ;;
            sync)
                cmd+="__sync"
                ;;
            undelete)
                cmd+="__undelete"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        apnotes)
            opts=" -h -V  --help --version  list edit sync delete undelete merge print backup new help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        
        apnotes__backup)
            opts=" -h -V  --help --version  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        apnotes__delete)
            _apnotes_get_notes delete
            return 0  
            ;;
        apnotes__edit)
            _apnotes_get_notes edit
            return 0
            ;;
        apnotes__help)
            opts=" -h -V  --help --version  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        apnotes__list)
            notes=$(apnotes list --names 2>&1 )
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${notes}") -- "${cur}") 
                return 0
            fi
            case "${prev}" in
                
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        apnotes__merge)

            _apnotes_get_notes merge
            return 0
            ;;
        apnotes__new)
            opts=" -f -h -V  --folder --help --version  <title> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --folder)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -f)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        apnotes__print)
            _apnotes_get_notes print
            return 0
            ;;
        apnotes__sync)
            opts=" -h -V  --help --version  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        apnotes__undelete)
            _apnotes_get_notes undelete
            return 0
            ;;
    esac
}

complete -o filenames -F _apnotes apnotes