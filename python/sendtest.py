from pythonosc import udp_client

client = udp_client.SimpleUDPClient("127.0.0.1", 13339)
#client.send_message("/subscribe", ["/test", 1, 2, 3])
client.send_message("/test", [1, "lol string", 1337.0, "/try_this", "whoah"])