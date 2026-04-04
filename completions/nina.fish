# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_nina_global_optspecs
	string join \n h/help
end

function __fish_nina_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_nina_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_nina_using_subcommand
	set -l cmd (__fish_nina_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c nina -n "__fish_nina_needs_command" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_needs_command" -f -a "apply"
complete -c nina -n "__fish_nina_needs_command" -f -a "back"
complete -c nina -n "__fish_nina_needs_command" -f -a "history"
complete -c nina -n "__fish_nina_needs_command" -f -a "go"
complete -c nina -n "__fish_nina_needs_command" -f -a "clean"
complete -c nina -n "__fish_nina_needs_command" -f -a "search"
complete -c nina -n "__fish_nina_needs_command" -f -a "install"
complete -c nina -n "__fish_nina_needs_command" -f -a "remove"
complete -c nina -n "__fish_nina_needs_command" -f -a "try"
complete -c nina -n "__fish_nina_needs_command" -f -a "list"
complete -c nina -n "__fish_nina_needs_command" -f -a "edit"
complete -c nina -n "__fish_nina_needs_command" -f -a "check"
complete -c nina -n "__fish_nina_needs_command" -f -a "diff"
complete -c nina -n "__fish_nina_needs_command" -f -a "status"
complete -c nina -n "__fish_nina_needs_command" -f -a "update"
complete -c nina -n "__fish_nina_needs_command" -f -a "upgrade"
complete -c nina -n "__fish_nina_needs_command" -f -a "log"
complete -c nina -n "__fish_nina_needs_command" -f -a "doctor"
complete -c nina -n "__fish_nina_needs_command" -f -a "help"
complete -c nina -n "__fish_nina_needs_command" -f -a "hello"
complete -c nina -n "__fish_nina_needs_command" -f -a "mood"
complete -c nina -n "__fish_nina_using_subcommand apply" -l on -r
complete -c nina -n "__fish_nina_using_subcommand apply" -l dry
complete -c nina -n "__fish_nina_using_subcommand apply" -l check
complete -c nina -n "__fish_nina_using_subcommand apply" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand back" -l on -r
complete -c nina -n "__fish_nina_using_subcommand back" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand history" -l on -r
complete -c nina -n "__fish_nina_using_subcommand history" -l tui
complete -c nina -n "__fish_nina_using_subcommand history" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand go" -l on -r
complete -c nina -n "__fish_nina_using_subcommand go" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand clean" -l on -r
complete -c nina -n "__fish_nina_using_subcommand clean" -l all
complete -c nina -n "__fish_nina_using_subcommand clean" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand search" -l on -r
complete -c nina -n "__fish_nina_using_subcommand search" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand install" -l on -r
complete -c nina -n "__fish_nina_using_subcommand install" -l no-apply
complete -c nina -n "__fish_nina_using_subcommand install" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand remove" -l on -r
complete -c nina -n "__fish_nina_using_subcommand remove" -l no-apply
complete -c nina -n "__fish_nina_using_subcommand remove" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand try" -l on -r
complete -c nina -n "__fish_nina_using_subcommand try" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand list" -l on -r
complete -c nina -n "__fish_nina_using_subcommand list" -l grep -r
complete -c nina -n "__fish_nina_using_subcommand list" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand edit" -l on -r
complete -c nina -n "__fish_nina_using_subcommand edit" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand check" -l on -r
complete -c nina -n "__fish_nina_using_subcommand check" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand diff" -l on -r
complete -c nina -n "__fish_nina_using_subcommand diff" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand status" -l on -r
complete -c nina -n "__fish_nina_using_subcommand status" -l all
complete -c nina -n "__fish_nina_using_subcommand status" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand update" -l on -r
complete -c nina -n "__fish_nina_using_subcommand update" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand upgrade" -l on -r
complete -c nina -n "__fish_nina_using_subcommand upgrade" -l check
complete -c nina -n "__fish_nina_using_subcommand upgrade" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand log" -l last -r
complete -c nina -n "__fish_nina_using_subcommand log" -l on -r
complete -c nina -n "__fish_nina_using_subcommand log" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand doctor" -l on -r
complete -c nina -n "__fish_nina_using_subcommand doctor" -l all
complete -c nina -n "__fish_nina_using_subcommand doctor" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand help" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand hello" -s h -l help -d 'Print help'
complete -c nina -n "__fish_nina_using_subcommand mood" -s h -l help -d 'Print help'
