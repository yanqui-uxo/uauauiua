import React, { useMemo, useState } from "react";

import { Checkbox, VisuallyHidden } from "@mantine/core";
import { useListState } from "@mantine/hooks";

import Wavesurfer from "wavesurfer.js";
import WavesurferPlayer from "@wavesurfer/react";

import { type Clip } from "./types";

function PlayerBase({
  players,
  wavesurfers,
}: {
  players: React.ReactElement[];
  wavesurfers: Wavesurfer[];
}) {
  return players;
}

export default function Player({ clips }: { clips: Clip[] }) {
  const [wavesurfers, wavesurfersHandlers] = useListState<Wavesurfer | null>(
    []
  );
  const players = useMemo(
    () =>
      clips.map((clip, i) => {
        const url = URL.createObjectURL(new Blob([new Uint8Array(clip.wav)]));
        return (
          <WavesurferPlayer
            key={url}
            url={url}
            peaks={clip.peaks}
            onReady={(ws) => wavesurfersHandlers.setItem(i, ws)}
            onInteraction={(ws) => ws.play()}
            onFinish={(ws) => ws.setTime(0)}
          />
        );
      }),
    [clips]
  );

  return wavesurfers.every(Boolean) ? (
    <PlayerBase players={players} wavesurfers={wavesurfers as Wavesurfer[]} />
  ) : (
    <>
      <p>Loading waveforms...</p>
      <VisuallyHidden>{players!}</VisuallyHidden>
    </>
  );
}
