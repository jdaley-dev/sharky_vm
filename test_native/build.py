import subprocess

PROGRAM_NAME = "native"
PROGRAM_FILES = ["main.c"]

def build():
    files = " ".join(PROGRAM_FILES)
    build_command = f"clang -shared -o {PROGRAM_NAME}.dll {files}"
    subprocess.run(build_command)

if __name__ == "__main__":
    build()