cat << EOF
Welcome to your development container. Happy coding!
EOF

export PS1="\[\e[36m\]\${OKTETO_NAMESPACE:-okteto}:\e[32m\]\${OKTETO_NAME:-dev} \[\e[m\]\W> "

PATH="/usr/local/cargo/bin:$PATH"

if ! shopt -oq posix; then
 if [ -f /usr/share/bash-completion/bash_completion ]; then
   . /usr/share/bash-completion/bash_completion
 elif [ -f /etc/bash_completion ]; then
   . /etc/bash_completion
 fi
fi

eval "$(rustup completions bash)"
eval "$(rustup completions bash cargo)"
