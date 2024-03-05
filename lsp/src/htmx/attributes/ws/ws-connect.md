Connect to <url> using the websocket protocol

<div hx-ext="ws" ws-connect="/chatroom">
    <div id="notifications"></div>
    <div id="chat_room">
        ...
    </div>
    <form id="form" ws-send>
        <input name="chat_message">
    </form>
</div>
