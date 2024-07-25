# Helpscript used during development to subscribe related services in the JackDAW ecosystem
# Should be in gitignore - not for public release
from pythonosc import udp_client

client = udp_client.SimpleUDPClient("127.0.0.1", 13339)

# jdw-sc subscriptions 
jdw_sc_port = 13331
jdw_seq_port = 14441
keyboard_port = 17777
client.send_message("/subscribe", ["/note_on_timed", "127.0.0.1", jdw_sc_port])
client.send_message("/subscribe", ["/note_on", "127.0.0.1", jdw_sc_port])
client.send_message("/subscribe", ["/bundle", "127.0.0.1", jdw_sc_port])
client.send_message("/subscribe", ["/c_set", "127.0.0.1", jdw_sc_port])
client.send_message("/subscribe", ["/play_sample", "127.0.0.1", jdw_sc_port])
client.send_message("/subscribe", ["/note_modify", "127.0.0.1", jdw_sc_port])
client.send_message("/subscribe", ["/read_scd", "127.0.0.1", jdw_sc_port])
client.send_message("/subscribe", ["/create_synthdef", "127.0.0.1", jdw_sc_port])
client.send_message("/subscribe", ["/load_sample", "127.0.0.1", jdw_sc_port])
client.send_message("/subscribe", ["/free_notes", "127.0.0.1", jdw_sc_port])
client.send_message("/subscribe", ["/clear_nrt", "127.0.0.1", jdw_sc_port])


client.send_message("/subscribe", ["/bundle", "127.0.0.1", jdw_seq_port])
client.send_message("/subscribe", ["/set_bpm", "127.0.0.1", jdw_seq_port])
client.send_message("/subscribe", ["/hard_stop", "127.0.0.1", jdw_seq_port])
client.send_message("/subscribe", ["/wipe_on_finish", "127.0.0.1", jdw_seq_port])

client.send_message("/subscribe", ["/set_bpm", "127.0.0.1", keyboard_port])
client.send_message("/subscribe", ["/keyboard_quantization", "127.0.0.1", keyboard_port])
client.send_message("/subscribe", ["/keyboard_args", "127.0.0.1", keyboard_port])
client.send_message("/subscribe", ["/keyboard_pad_args", "127.0.0.1", keyboard_port])
client.send_message("/subscribe", ["/keyboard_pad_samples", "127.0.0.1", keyboard_port])
client.send_message("/subscribe", ["/keyboard_pad_pack", "127.0.0.1", keyboard_port])
client.send_message("/subscribe", ["/keyboard_letter_index", "127.0.0.1", keyboard_port])
client.send_message("/subscribe", ["/keyboard_mode_synth", "127.0.0.1", keyboard_port])
client.send_message("/subscribe", ["/keyboard_mode_sampler", "127.0.0.1", keyboard_port])
client.send_message("/subscribe", ["/keyboard_instrument_name", "127.0.0.1", keyboard_port])
client.send_message("/subscribe", ["/loop_started", "127.0.0.1", keyboard_port])

# Test port 
client.send_message("/subscribe", ["/sequencer_tick_test", "127.0.0.1", 15454])


jdw_sample_port = 12367
client.send_message("/subscribe", ["/play_sample", "127.0.0.1", jdw_sample_port])
