export const soundAssets = {
  send: '/assets/sounds/send.mp3',
  receive: '/assets/sounds/receive.mp3',
  typing: '/assets/sounds/typing.mp3',
  notification: '/assets/sounds/notification.mp3',
  click: '/assets/sounds/click.mp3',
  hover: '/assets/sounds/hover.mp3',
  success: '/assets/sounds/success.mp3',
  error: '/assets/sounds/error.mp3'
} as const;

// Type for sound names
export type SoundName = keyof typeof soundAssets;
