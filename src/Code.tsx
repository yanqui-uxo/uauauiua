import {useState} from "react"

import taurpc from "./proxy";

function PlayerButtons({samples}: {samples: {[v: string]: number[]}}) {
	return <>
		<button onClick={async () => {
			await taurpc.load_stack_sample();
			await taurpc.play();
		}}>Play stack sample</button>
		{Object.entries(samples).map(
			([k]) => <button key={k} onClick={async () => {
				await taurpc.load_var_sample(k);
				await taurpc.play();
			}}>
				Play {k}
			</button>
		)}
	</>
}

export default function Code() {
	const [code, setCode] = useState("");
	const [samples, setSamples] = useState({});
	return <>
		<textarea value={code} onChange={e => setCode(e.target.value)} />
		<button onClick={async () => {
			const newCode = await taurpc.format_code(code);
			setCode(newCode);
			await taurpc.run_code(newCode);
			setSamples(await taurpc.var_samples());
		}}>Run code</button>
		<PlayerButtons samples={samples} />

	</>
}