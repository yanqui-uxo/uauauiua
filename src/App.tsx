import "@mantine/core/styles.css";

import { useEffect, useState } from "react";

import { MantineProvider } from "@mantine/core";

import taurpc from "./proxy";
import { type AudioData } from "./types";
import VarPlayer from "./VarPlayer";
import StackPlayer from "./StackPlayer";

export default function App() {
  const [audioData, setAudioData] = useState<AudioData | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    taurpc.file_loaded.on((data) => {
      setAudioData(data);
      setError("");
    });
    taurpc.error.on(setError);
  }, []);

  return (
    <MantineProvider defaultColorScheme="dark">
      {audioData && (
        <>
          <StackPlayer clips={audioData.stackClips} />
          <VarPlayer clips={audioData.varClips} />
        </>
      )}
      <p>{error}</p>
    </MantineProvider>
  );
}
