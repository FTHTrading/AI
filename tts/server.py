"""
Genesis Narrator — Piper VITS neural TTS server.

FastAPI service wrapping Piper for high-quality narration.
Voices: en_US-lessac-high (warm narrator), en_US-hfc_male-medium (deep male).

Usage:
    python tts/server.py
    # → http://localhost:8770

Endpoints:
    POST /synthesize   — text → WAV audio
    GET  /voices       — list available voices
    GET  /health       — status check
"""

import io
import os
import time
import wave
import struct
import math
from pathlib import Path
from typing import Optional

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import Response
from pydantic import BaseModel, Field
from piper import PiperVoice
from piper.config import SynthesisConfig

# ── Paths ────────────────────────────────────────────────────────
VOICES_DIR = Path(__file__).parent / "voices"

# ── Voice registry ───────────────────────────────────────────────
VOICE_CONFIGS = {
    "narrator": {
        "model": "en_US-lessac-high.onnx",
        "description": "Professional narrator — warm, clear, expressive",
        "defaults": {
            "length_scale": 1.12,   # slightly slower for gravitas
            "noise_scale": 0.45,    # clean articulation
            "noise_w_scale": 0.55,  # natural duration variance
        },
    },
    "deep": {
        "model": "en_US-hfc_male-medium.onnx",
        "description": "Deep male — Morgan Freeman register",
        "defaults": {
            "length_scale": 1.20,   # slower, deliberate
            "noise_scale": 0.40,    # very clean
            "noise_w_scale": 0.50,  # controlled pacing
        },
    },
}

# ── Prosody profiles per content type ────────────────────────────
PROSODY_PROFILES = {
    "narrative":  {"length_scale": 1.12, "noise_scale": 0.45, "noise_w_scale": 0.55},
    "heading":    {"length_scale": 1.05, "noise_scale": 0.50, "noise_w_scale": 0.45},
    "quote":      {"length_scale": 1.18, "noise_scale": 0.40, "noise_w_scale": 0.60},
    "epitaph":    {"length_scale": 1.30, "noise_scale": 0.35, "noise_w_scale": 0.65},
    "science":    {"length_scale": 1.08, "noise_scale": 0.50, "noise_w_scale": 0.50},
    "table":      {"length_scale": 1.00, "noise_scale": 0.55, "noise_w_scale": 0.45},
    "chronicle":  {"length_scale": 1.15, "noise_scale": 0.42, "noise_w_scale": 0.58},
    "calm":       {"length_scale": 1.15, "noise_scale": 0.42, "noise_w_scale": 0.55},
    "tense":      {"length_scale": 1.05, "noise_scale": 0.55, "noise_w_scale": 0.50},
    "critical":   {"length_scale": 0.98, "noise_scale": 0.60, "noise_w_scale": 0.45},
    "recovering": {"length_scale": 1.12, "noise_scale": 0.45, "noise_w_scale": 0.55},
    "dead":       {"length_scale": 1.35, "noise_scale": 0.30, "noise_w_scale": 0.70},
}


# ── Loaded voice cache ──────────────────────────────────────────
_voices: dict[str, PiperVoice] = {}


def get_voice(name: str) -> PiperVoice:
    if name not in _voices:
        cfg = VOICE_CONFIGS.get(name)
        if not cfg:
            raise HTTPException(404, f"Unknown voice: {name}")
        model_path = VOICES_DIR / cfg["model"]
        if not model_path.exists():
            raise HTTPException(500, f"Voice model not found: {model_path}")
        _voices[name] = PiperVoice.load(str(model_path))
    return _voices[name]


# ── App ──────────────────────────────────────────────────────────
app = FastAPI(title="Genesis Narrator", version="1.0.0")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)


class SynthRequest(BaseModel):
    text: str = Field(..., min_length=1, max_length=5000)
    voice: str = Field("narrator", description="Voice name: narrator | deep")
    content_type: Optional[str] = Field(None, description="Prosody profile key")
    speed: float = Field(1.0, ge=0.5, le=2.0, description="Speed multiplier")
    silence_ms: int = Field(300, ge=0, le=3000, description="Silence appended at end (ms)")


def add_silence(audio_bytes: bytes, sample_rate: int, ms: int) -> bytes:
    """Append silence (zero samples) to audio."""
    if ms <= 0:
        return audio_bytes
    n_samples = int(sample_rate * ms / 1000)
    silence = b'\x00\x00' * n_samples  # 16-bit silence
    return audio_bytes + silence


@app.post("/synthesize")
async def synthesize(req: SynthRequest):
    t0 = time.time()

    voice = get_voice(req.voice)
    voice_cfg = VOICE_CONFIGS[req.voice]
    sample_rate = voice.config.sample_rate

    # Build synthesis config from voice defaults + prosody profile + speed
    defaults = voice_cfg["defaults"].copy()
    if req.content_type and req.content_type in PROSODY_PROFILES:
        defaults.update(PROSODY_PROFILES[req.content_type])

    # Apply speed multiplier to length_scale (higher length_scale = slower)
    length_scale = defaults["length_scale"] / req.speed

    syn_cfg = SynthesisConfig(
        length_scale=length_scale,
        noise_scale=defaults["noise_scale"],
        noise_w_scale=defaults["noise_w_scale"],
        volume=1.0,
    )

    # Synthesize to WAV in memory
    buf = io.BytesIO()
    with wave.open(buf, "wb") as wf:
        voice.synthesize_wav(req.text, wf, syn_config=syn_cfg)

    wav_data = buf.getvalue()

    # Add trailing silence if requested
    if req.silence_ms > 0:
        # WAV header is 44 bytes, audio data follows
        header = wav_data[:44]
        audio = wav_data[44:]
        audio = add_silence(audio, sample_rate, req.silence_ms)
        # Rebuild WAV with correct sizes
        buf2 = io.BytesIO()
        with wave.open(buf2, "wb") as wf:
            wf.setnchannels(1)
            wf.setsampwidth(2)
            wf.setframerate(sample_rate)
            wf.writeframes(audio)
        wav_data = buf2.getvalue()

    elapsed = time.time() - t0
    audio_duration = (len(wav_data) - 44) / (sample_rate * 2)

    return Response(
        content=wav_data,
        media_type="audio/wav",
        headers={
            "X-Synthesis-Time": f"{elapsed:.3f}",
            "X-Audio-Duration": f"{audio_duration:.2f}",
            "X-Voice": req.voice,
            "X-Sample-Rate": str(sample_rate),
        }
    )


@app.get("/voices")
async def list_voices():
    return {
        name: {
            "description": cfg["description"],
            "defaults": cfg["defaults"],
            "loaded": name in _voices,
        }
        for name, cfg in VOICE_CONFIGS.items()
    }


@app.get("/prosody")
async def list_prosody():
    return PROSODY_PROFILES


@app.get("/health")
async def health():
    models_present = {
        name: (VOICES_DIR / cfg["model"]).exists()
        for name, cfg in VOICE_CONFIGS.items()
    }
    return {
        "status": "ok",
        "voices_dir": str(VOICES_DIR),
        "models": models_present,
    }


if __name__ == "__main__":
    import uvicorn
    print("═" * 60)
    print("  Genesis Narrator — Piper VITS Neural TTS")
    print(f"  Voices dir: {VOICES_DIR}")
    print(f"  Available:  {', '.join(VOICE_CONFIGS.keys())}")
    print("═" * 60)

    # Pre-load default voice
    print("Loading narrator voice...")
    get_voice("narrator")
    print("Narrator loaded. Loading deep voice...")
    get_voice("deep")
    print("All voices loaded.")
    print("═" * 60)

    uvicorn.run(app, host="127.0.0.1", port=8770, log_level="info")
