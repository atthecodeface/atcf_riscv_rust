# Install rust

'''
install_dir = /rust
mkdir ${install_dir}

export CARGO_HOME=${install_dir}/cargo
export RUSTUP_HOME=${install_dir}/rustup
export PATH=${CARGO_HOME}/bin:$PATH

cd ${install_dir}
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup
chmod +x ./rustup
./rustup -y

rustup target add riscv32imc-unknown-none-elf
'''
