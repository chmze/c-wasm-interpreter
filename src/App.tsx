import "./App.css";
import { greet } from "c-wasm-interpreter";

function App() {
    greet();

    return (
        <div>Hello</div>
    );
};

export default App;
