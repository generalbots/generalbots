// Player module JavaScript
document.addEventListener('DOMContentLoaded', () => {
  const playerContainer = document.querySelector('.player-container');
  const videoContainer = document.querySelector('.video-container');
  const playBtn = document.querySelector('.play-btn');
  const progressSlider = document.querySelector('.progress-slider');
  const volumeSlider = document.querySelector('.volume-slider');
  const timeDisplay = document.querySelector('.time-display');

  // Create video element
  const video = document.createElement('video');
  video.src = ''; // Will be set when loading media
  video.controls = false;
  videoContainer.appendChild(video);

  // Play/Pause toggle
  playBtn.addEventListener('click', () => {
    if (video.paused) {
      video.play();
      playBtn.textContent = 'Pause';
    } else {
      video.pause();
      playBtn.textContent = 'Play';
    }
  });

  // Update progress slider
  video.addEventListener('timeupdate', () => {
    const progress = (video.currentTime / video.duration) * 100;
    progressSlider.value = progress || 0;
    updateTimeDisplay();
  });

  // Seek video
  progressSlider.addEventListener('input', () => {
    const seekTime = (progressSlider.value / 100) * video.duration;
    video.currentTime = seekTime;
  });

  // Volume control
  volumeSlider.addEventListener('input', () => {
    video.volume = volumeSlider.value / 100;
  });

  // Format time display
  function updateTimeDisplay() {
    const formatTime = (seconds) => {
      const mins = Math.floor(seconds / 60);
      const secs = Math.floor(seconds % 60);
      return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
    };

    timeDisplay.textContent = `${formatTime(video.currentTime)} / ${formatTime(video.duration)}`;
  }

  // Load media (to be called externally)
  window.loadMedia = (src) => {
    video.src = src;
    video.load();
  };
});
