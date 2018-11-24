let uri = "ws://127.0.0.1:3012/";
let webSocket = null;

function init() {
  open();
}

function open() {
  if (webSocket == null) {
    // WebSocket の初期化
    webSocket = new WebSocket(uri);
    // イベントハンドラの設定
    webSocket.onopen = onOpen;
    webSocket.onmessage = onMessage;
    webSocket.onclose = onClose;
    webSocket.onerror = onError;
  }
}

function onOpen(event) {
  chat("Connected.");
}

function onMessage(event) {
  if (event && event.data) {
    let obj = document.getElementById('board');
    $(obj).data('qfcode',event.data);
    chat(event.data);
    console.log("hoge");
    createWholeBoard(0,obj);
  }
}

function onError(event) {
  chat("error occured");
}

function onClose(event) {
  chat("切断しました。3秒後に再接続します。(" + event.code + ")");
  webSocket = null;
  setTimeout("open()", 3000);
}

function chat(message) {
  let chats = $("[data-name='chat']").find("div");
  while (chats.length >= 100) {
    chats = chats.last().remove();
  }
  let msgtag = $("<div>").text(message);
  $("[data-name='chat']").prepend(msgtag);
}

$(init);

