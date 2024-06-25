import "@mantine/core/styles.css";

import { useState } from "react";

import { Button, MantineProvider, Textarea } from "@mantine/core";

import taurpc from "./proxy";
import StackPlayer from "./StackPlayer";
import { type AudioData } from "./types";
import VarPlayer from "./VarPlayer";

export default function App() {
  const [code, setCode] = useState('P ~ "../prelude.ua"\n');
  const [audioData, setAudioData] = useState<AudioData | null>(null);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState("");

  return (
    <MantineProvider forceColorScheme="dark">
      <Textarea
        value={code}
        onChange={(e) => setCode(e.target.value)}
        resize="both"
      />
      <Button
        onClick={async () => {
          setRunning(true);
          setError("");
          try {
            const newCode = await taurpc.format_code(code);
            setCode(newCode);
            setAudioData(await taurpc.run_code(newCode));
          } catch (e) {
            setError(`Error: ${e} (time: ${Date.now()})`);
          }
          setRunning(false);
        }}
      >
        Run code
      </Button>
      {running && <p>Running...</p>}
      {audioData && !running && (
        <>
          <StackPlayer clips={audioData.stackClips} />
          <VarPlayer clips={audioData.varClips} />
        </>
      )}
      <p>{error}</p>
    </MantineProvider>
  );
}
