import argparse
import subprocess
import os

parser = argparse.ArgumentParser()
parser.add_argument("--meta_node_address", type=str, default="127.0.0.1:5690")
parser.add_argument("--compute_node_number", type=int, default=1)
parser.add_argument("--compute_node_port", type=int, default=6690)
parser.add_argument("--exe", type=str, default="bin/fake-compute-node")
args = parser.parse_args()

exe = args.exe
meta_address = args.meta_node_address
cn_number = args.compute_node_number
cn_port_base = args.compute_node_port

print("Press any key to exit..")

for cn_idx in range(0, cn_number):
    cn_address = "127.0.0.1:%d" % (cn_port_base + cn_idx)
    print("start cn %s" % cn_address)
    subprocess.Popen([exe, "--meta-address=%s" % meta_address, "--host-address=%s" % cn_address], stdin=subprocess.PIPE)

input()
os.system("kill -9 `pidof fake-compute-node`")
