import { useMemo } from "react";

import WavesurferPlayer, { WavesurferProps } from "@wavesurfer/react";

import { type Clip } from "./types";

export default function CustomWavesurferPlayer({
  clip,
  ...rest
}: {
  clip: Clip;
} & WavesurferProps) {
  const url = useMemo(
    () => URL.createObjectURL(new Blob([new Uint8Array(clip.wav)])),
    [clip]
  );
  return (
    <WavesurferPlayer
      url={url}
      onInteraction={(ws) => ws.play()}
      onFinish={(ws) => ws.setTime(0)}
      dragToSeek={true}
      {...rest}
    />
  );
}
