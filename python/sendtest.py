from pythonosc import udp_client

# Send a message to the router
client = udp_client.SimpleUDPClient("127.0.0.1", 13339)
client.send_message("/test", [1, "A string", 1337.0, "/try_this", "whoah"])