FROM okteto/rust:1.47.0

RUN apt update && apt upgrade -y && apt install -y --no-install-recommends bash-completion locales locales-all less vim moreutils && \
    rustup self update && rustup default 1.60 && rustup toolchain uninstall 1.47.0 && rustup component add rustfmt
