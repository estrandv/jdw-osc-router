# Helpscript used during development to subscribe related services in the JackDAW ecosystem
# Should be in gitignore - not for public release
from pythonosc import udp_client

client = udp_client.SimpleUDPClient("127.0.0.1", 13339)

# jdw-sc subscriptions 
jdw_sc_port = 13331
client.send_message("/subscribe", ["/note_on_timed", "127.0.0.1", jdw_sc_port])
client.send_message("/subscribe", ["/bundle", "127.0.0.1", jdw_sc_port])
client.send_message("/subscribe", ["/play_sample", "127.0.0.1", jdw_sc_port])

jdw_seq_port = 14441
client.send_message("/subscribe", ["/bundle", "127.0.0.1", jdw_seq_port])

jdw_sample_port = 12367
client.send_message("/subscribe", ["/play_sample", "127.0.0.1", jdw_sample_port])
