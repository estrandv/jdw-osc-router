# OSC Router
Minimalist CLI application built to allow dynamic routing of OSC messages to any number of subscribers. 
Takes OSC messages such as ["/subscribe", "/s_new", 5567] to route any incoming such messages to the subscriber.

See the python subdir for example usage. sendtest.py and subscribertest.py can work in tandem to demonstrate message flow:
    1. Start the router with cargo run
    2. Start subscribertest.py (note that this will immediately send a /subscribe message to the router)
    3. Use sendtest.py to send messages to the subscriber via the router

# Functions

### /subscribe or /unsubscribe
- Arg0: The address to subscribe/unsubscribe to/from, e.g. "/my_func"
- Arg1: The port of the subscriber
- Example: ["/subscribe", "/test", 13332] (This will send all /test messages to the subscriber on its inferred ip and provided port)
- Explanation: A subscribing osc server that wants to receive messages for the given OSC address (arg0) must provide both
    this OSC address and its own port. The ip of the server is inferred from the udp call. Once a subscriber is registered,
    all messages for the subscribed address will be cloned and sent to that subscriber immediately upon being received by
    the router. 