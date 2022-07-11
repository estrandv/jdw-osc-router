# Helpscript used during development to subscribe related services in the JackDAW ecosystem
# Should be in gitignore - not for public release
from pythonosc import udp_client

client = udp_client.SimpleUDPClient("127.0.0.1", 13339)

# Note supercollider.rs ports in jdw-sc project
client.send_message("/subscribe", ["/s_new_timed_gate", "127.0.0.1", 13331])