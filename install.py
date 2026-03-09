import os

#NOTE Написано в 2 часа ночи.
#COLORES FOR OUTPUT

GREEN = "\033[92m";
RED = "\033[91m";
YELLOW = "\033[93m";
RESET = "\033[0m";

def print_step(msg):
    print(f"{GREEN}==>{RESET} {msg}");

def print_error(msg):
    print(f"{RED}Error:{RESET} {msg}");

def print_warning(msg):
    print(f"{YELLOW}Warning:{RESET} {msg}");

print_step("Start building project");
os.system("cargo build --release");

user_input = input("Copy binary to /usr/bin/arrowc? (y/N): ").strip().lower();
if user_input in ('y', 'yes'):
    os.system("sudo cp target/release/ArrowRust /usr/bin/arrowc");
    print_step("Binary installed to /usr/bin/arrowc");
else:
    # Альтернатива: предложить скопировать в ~/.local/bin
    user_bin = os.path.expanduser("~/.local/bin");
    os.makedirs(user_bin, exist_ok=True);
    os.system(f"cp target/release/ArrowRust {user_bin}/arrowc");
    print_step(f"Binary installed to {user_bin}/arrowc (add this dir to PATH if needed)");

arrow_path = input("How dir use for libs? enter in format /path/to/dir \n");

print_step("Copying std");
os.system(f"sudo mkdir -p {arrow_path}/std");
os.system(f'sudo cp -RT std/ "{arrow_path}/std"');

print_warning("Set $ARROWPATH manually for your shell based on the path you entered.");