import "./App.css";
import Main from "./routes/Main";
import SnipOverlay from "./routes/SnipOverlay";
import { createHashRouter, RouterProvider } from "react-router";

const router = createHashRouter([
  { path: "/", element: <Main /> },
  { path: "/snip", element: <SnipOverlay /> },
]);

function App() {
  return <RouterProvider router={router} />
}

export default App;
