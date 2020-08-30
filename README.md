## What is this?

This is the monitoring and central configuration server for https://github.com/course-correction/roc-droid.

Lets describe what this server does:

1. The server uses dns-sd to announce its hostname and port on the network.

2. The android app discovers the service and creates a connection to the server.

3. The now established tcp connection is used to exchange commands and status information between
   server and client. This can be configuration for where the android app should stream its audio
   to, the battery level or log messages from the client.

4. The server displays client state in the terminal for all connected clients.

5. If the client notifies the server about a configuration change, the server persist this
   information into a json file. If this file is modified by an external editor, the server takes
   the change and sends it to the client.

## Regarding some architecture choices

When a configuration file is changed, the changed setting is send to the android app. At this moment
the server still thinks of the previous value of being active. It is expected that the client
sends the new status back to the server to confirm that it has accepted the change. Now the same 
execution path is used as if the client initiated the change. Next the information in the text user
interface is updated.

Upon establishing a connection a client should send a hello message containing a unique device
identifier. It is mainly used to give the persisted json config file a name. As such it should be
compatible with common file systems. Please also note. This enforces that the client will only
receive settings when it has identified itself. There is no fallback configuration fo unknown
clients (You only have to copy an existing config file to the name of the new client id.).

## Dependencies

Mac works out of the box. Fedora 32 needs `avahi-compat-libdns_sd`.

## License

Dunno yet. Ask me if relevant for you.