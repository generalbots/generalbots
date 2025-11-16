// Player module JavaScript
document.addEventListener('DOMContentLoaded', () => {
  const ready = setInterval(() => {
    const container = document.querySelector('.player-container');
    if (!container) return;
    clearInterval(ready);
    const videoContainer = document.querySelector('.video-container');
    const playBtn = document.querySelector('.play-btn');
    const progressSlider = document.querySelector('.progress-slider');
    const volumeSlider = document.querySelector('.volume-slider');
    const timeDisplay = document.querySelector('.time-display');
    const ensure = setInterval(() => {
      const container = document.querySelector('.video-container');
      if (container) {
        clearInterval(ensure);
        const video = document.createElement('video');
        video.src = '';
        video.controls = false;
        container.appendChild(video);
        setupControls(video);
      }
    }, 50);
    function setupControls(video) {
      const playBtn = document.querySelector('.play-btn');
      const progressSlider = document.querySelector('.progress-slider');
      const volumeSlider = document.querySelector('.volume-slider');
      const timeDisplay = document.querySelector('.time-display');
      playBtn.addEventListener('click', () => {
        if (video.paused) {
          video.play();
          playBtn.textContent = 'Pause';
        } else {
          video.pause();
          playBtn.textContent = 'Play';
        }
      });
      video.addEventListener('timeupdate', () => {
        const progress = (video.currentTime / video.duration) * 100;
        progressSlider.value = progress || 0;
        updateTimeDisplay(video, timeDisplay);
      });
      progressSlider.addEventListener('input', () => {
        const seekTime = (progressSlider.value / 100) * video.duration;
        video.currentTime = seekTime;
      });
      volumeSlider.addEventListener('input', () => {
        video.volume = volumeSlider.value / 100;
      });
      window.loadMedia = (src) => {
        video.src = src;
        video.load();
      };
    }
    function updateTimeDisplay(video, display) {
      const formatTime = (seconds) => {
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
      };
      display.textContent = `${formatTime(video.currentTime)} / ${formatTime(video.duration)}`;
    }
  });
});
