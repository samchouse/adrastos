FROM ubuntu:latest

RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y curl git nano wget build-essential zsh libssl-dev pkg-config

SHELL ["/bin/zsh", "-c"]
RUN chsh -s $(which zsh)
RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
RUN sed -i 's/ZSH_THEME=/\# \0/g' ~/.zshrc

RUN git clone https://github.com/zsh-users/zsh-syntax-highlighting.git ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/zsh-syntax-highlighting
RUN sed -i 's/plugins=(git/\0 zsh-syntax-highlighting/g' ~/.zshrc

RUN curl -sS https://starship.rs/install.sh | sh -s -- -y
RUN echo "\neval \"\$(starship init zsh)\"" >> ~/.zshrc
RUN mkdir ~/.config
RUN starship preset plain-text-symbols -o ~/.config/starship.toml

RUN curl https://rtx.pub/install.sh | sh
RUN echo -e "\neval \"\$(/root/.local/share/rtx/bin/rtx activate -s zsh)\"" >> ~/.zshrc
ENV PATH ${PATH}:/root/.local/share/rtx/bin
RUN rtx use --global node@20

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN rustup toolchain install nightly

RUN rtx exec nodejs@latest -- npm i -g yarn taze
RUN cargo install cargo-edit cargo-update cargo-watch topgrade
