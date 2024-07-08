import { useMemo, useRef, useState } from "react";

import { Button, Checkbox, VisuallyHidden } from "@mantine/core";
import { useDidUpdate, useListState } from "@mantine/hooks";

import stringify from "json-stable-stringify";
import Wavesurfer from "wavesurfer.js";

import CustomWavesurferPlayer from "./CustomWavesurferPlayer";
import { type Clip } from "./types";

function StackPlayerBase({ clips }: { clips: Clip[] }) {
  const [wavesurfers, wavesurfersHandlers] = useListState<Wavesurfer | null>(
    Array(clips.length).fill(null)
  );
  const wavesurfersRef = useRef<(Wavesurfer | null)[]>(wavesurfers);
  const [checkedIndices, setCheckedIndices] = useState(clips.map((_, i) => i));
  const checkedIndicesRef = useRef(checkedIndices);
  const players = useMemo(
    () =>
      clips.map((clip, i) => (
        <CustomWavesurferPlayer
          key={stringify(clip)}
          clip={clip}
          interact={checkedIndices.includes(i)}
          onReady={(ws) => wavesurfersHandlers.setItem(i, ws)}
          onInteraction={(_, t) => {
            checkedIndicesRef.current.forEach((i) => {
              console.log(wavesurfers);
              wavesurfersRef.current[i]!.setTime(t);
              wavesurfersRef.current[i]!.play();
            });
          }}
          onFinish={(ws) => ws.setTime(0)}
        />
      )),
    [clips]
  );

  useDidUpdate(() => {
    wavesurfersRef.current = wavesurfers;
  }, [wavesurfers]);
  useDidUpdate(() => {
    checkedIndicesRef.current = checkedIndices;
    wavesurfers.forEach((ws, i) =>
      ws!.setOptions({ interact: checkedIndices.includes(i) })
    );
  }, [checkedIndices]);

  // the players are rendered hidden so the wavesurfers can initialize
  return wavesurfers.length > 0 && wavesurfers.every(Boolean) ? (
    <>
      <Checkbox.Group
        value={checkedIndices.map((i) => i.toString())}
        onChange={(vs) => {
          wavesurfersRef.current.forEach((ws) => {
            ws!.pause();
            ws!.setTime(0);
          });
          setCheckedIndices(vs.map((v) => parseInt(v)));
        }}
      >
        {clips.map((_, i) => (
          <Checkbox key={i} value={i.toString()} label={i.toString()} />
        ))}
      </Checkbox.Group>
      {players}
      <Button
        onClick={() => checkedIndices.forEach((i) => wavesurfers[i]!.play())}
      >
        Play
      </Button>
      <Button
        onClick={() => checkedIndices.forEach((i) => wavesurfers[i]!.pause())}
      >
        Pause
      </Button>
    </>
  ) : (
    <>
      <p>Loading waveforms...</p>
      <VisuallyHidden>{players}</VisuallyHidden>
    </>
  );
}

export default function StackPlayer({ clips }: { clips: Clip[] }) {
  const key = useMemo(() => stringify(clips), [clips]);
  return <StackPlayerBase key={key} clips={clips} />;
}
