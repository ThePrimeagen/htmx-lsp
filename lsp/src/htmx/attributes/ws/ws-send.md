Send a message to the nearest websocket based on the trigger value for the element

<div hx-ext="ws" ws-connect="/chatroom">
    <div id="notifications"></div>
    <div id="chat_room">
        ...
    </div>
    <form id="form" ws-send>
        <input name="chat_message">
    </form>
</div>
