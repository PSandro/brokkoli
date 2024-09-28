function display_humidity() {
  const url = ((location.protocol !== "https:")?"ws://":"wss://") + window.location.host + "/ws";
  let ws = new WebSocket(url);
  ws.onmessage = function(msg) {
    const data = JSON.parse(msg.data);
    document.getElementById('humidity').innerText = data.humidity.toFixed(1);
    document.getElementById('temperature').innerText = data.temperature.toFixed(1);
  }
}

function init_audio() {
  let audio = document.getElementById('audioPlayer');
  let playPauseButton = document.getElementById('playPauseButton');
  playPauseButton.addEventListener('click', function() {
    if (audio.paused) {
      audio.play();
      playPauseButton.textContent = 'Pause';
    } else {
      audio.pause();
      playPauseButton.textContent = 'Play';
    }
  });
}

window.onload = () => {
  init_audio();
  display_humidity();
}
