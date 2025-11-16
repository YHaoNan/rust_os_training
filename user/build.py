import os

# bin path
bin_path = 'src/bin'
# linker.ld file
linker_path = 'src/linker.ld'

# read all bin.rs
bin_list = os.listdir('src/bin')

# define the first app base address
BASE_ADDRESS = 0x80400000
# define the app size
APP_SIZE = 0x20000

# save the original content of the linker.ld
original_linkerld = open(linker_path, 'r+').read()

# for each bin.rs
for bin in bin_list:

    path = bin_path + '/' + bin
    print(f"[build.py] Compiling user program [{bin}] to [{BASE_ADDRESS:#x}]")

    with open(path, 'r+') as f:
        # generate it specific linker.ld (with specific BASE_ADDRESS)
        specific_linkerld = original_linkerld.replace('0x80400000', hex(BASE_ADDRESS))
        open(linker_path, 'w+').write(specific_linkerld)
        app = bin[:bin.find('.')]
        os.system(f"cargo build --bin {app} --release") # build with cargo
        print(f"[build.py] {app} build successed!")

    BASE_ADDRESS += APP_SIZE


# restore the original linker.ld
open(linker_path, 'w+').write(original_linkerld)