import json
import unittest

from pydantic import ValidationError

from src.schemas.generation import MusicGenerateRequest
from src.services.music_service import MusicService


class MusicRequestTests(unittest.TestCase):
    def test_simple_mode_requires_description(self):
        with self.assertRaises(ValidationError):
            MusicGenerateRequest(description="")

    def test_instrumental_discards_lyrics(self):
        request = MusicGenerateRequest(
            description="A warm soul instrumental", instrumental=True, lyrics="words"
        )
        self.assertEqual(request.lyrics, "")


class MusicResultTests(unittest.TestCase):
    def test_audio_path_accepts_windows_absolute_and_rejects_traversal(self):
        self.assertTrue(MusicService._valid_audio_path(r"C:\tmp\api_audio\song.wav"))
        self.assertFalse(MusicService._valid_audio_path(r"C:\tmp\..\secret.wav"))

    def test_parse_tracks_normalizes_acestep_audio_url(self):
        raw = json.dumps(
            [
                {
                    "file": "/v1/audio?path=%2Ftmp%2Fapi_audio%2Fsong.mp3",
                    "prompt": "warm analog soul",
                    "lyrics": "[Verse] Hello",
                    "metas": {
                        "bpm": 92,
                        "duration": 61.5,
                        "keyscale": "A Minor",
                        "timesignature": "4",
                    },
                    "seed_value": "42",
                    "dit_model": "acestep-v15-turbo",
                }
            ]
        )

        tracks = MusicService._parse_tracks(raw)

        self.assertEqual(len(tracks), 1)
        self.assertEqual(tracks[0].audio_path, "/tmp/api_audio/song.mp3")
        self.assertEqual(tracks[0].bpm, 92)
        self.assertEqual(tracks[0].model, "acestep-v15-turbo")


if __name__ == "__main__":
    unittest.main()
