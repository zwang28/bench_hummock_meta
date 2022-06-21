import argparse
import subprocess
import os

parser = argparse.ArgumentParser()
parser.add_argument("--meta_node_address", type=str, default="127.0.0.1:5690")
parser.add_argument("--exe", type=str, default="bin/fake-barrier-manager")
args = parser.parse_args()

exe = args.exe
meta_address = args.meta_node_address

print("Press any key to exit..")

print("start barrier manager")
subprocess.Popen([exe, "--meta-address=%s" % meta_address], stdin=subprocess.PIPE)

input()
os.system("kill -9 `pidof fake-barrier-manager`")
