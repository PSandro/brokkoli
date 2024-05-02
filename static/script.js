function display_humidity() {
  const url = ((location.protocol !== "https:")?"ws://":"wss://") + window.location.host + "/ws";
  let ws = new WebSocket(url);
  ws.onmessage = function(msg) {
    document.getElementById('humidity').innerText = msg.data;
  }
}

window.onload = () => {display_humidity()}
