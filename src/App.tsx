import "@mantine/core/styles.css";

import { useState } from "react";

import { Button, MantineProvider, Textarea } from "@mantine/core";

import taurpc from "./proxy";
import Player from "./Player";
import { type Clip } from "./types";

export default function App() {
  const [code, setCode] = useState('P ~ "../prelude.ua"\n');
  const [clips, setClips] = useState<Clip[] | null>(null);
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
          try {
            const newCode = await taurpc.format_code(code);
            setCode(newCode);
            const data = await taurpc.run_code(newCode);
            setClips(data.stackClips);
          } catch (e) {
            setError(`Error: ${e} (time: ${Date.now()})`);
          }
        }}
      >
        Run code
      </Button>
      {clips && <Player clips={clips} />}
      <p>{error}</p>
    </MantineProvider>
  );
}
