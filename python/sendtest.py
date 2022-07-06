from pythonosc import udp_client

client = udp_client.SimpleUDPClient("127.0.0.1", 13339)

client.send_message("/test", ["test", 1, 2, 3])