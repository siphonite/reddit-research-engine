import { useState } from "react";
import "./App.css";

// Fix: Match the actual filenames (case-sensitive)
import Navbar from "./components/Navbar";
import UrlInput from "./components/UrlInput";
import ResultCard from "./components/ResultCard";
import Footer from "./components/Footer";

function App() {
  const [redditUrl, setRedditUrl] = useState("");
  const [result, setResult] = useState(null);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const handleGenerate = async () => {
    if (!redditUrl.trim()) {
      setError("Please enter a Reddit post link.");
      return;
    }

    setError("");
    setLoading(true);

    try {
      const response = await fetch("http://127.0.0.1:3000/analyze_post", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url: redditUrl }),
      });

      const data = await response.json();

      if (!response.ok) {
        setError(data.error || "Something went wrong.");
      } else {
        setResult(data);
      }
    } catch (err) {
      setError("Server unreachable.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      <Navbar />

      <div className="container">
        <h1 className="title">Turn Any Reddit Post Into a Startup Idea</h1>
        <p className="subtitle">
          Paste a Reddit post link and get a fully structured startup idea with
          problem → solution → business model.
        </p>

        <UrlInput
          redditUrl={redditUrl}
          setRedditUrl={setRedditUrl}
          loading={loading}
          handleGenerate={handleGenerate}
          error={error}
        />

        {result && <ResultCard result={result} />}
      </div>

      <Footer />
    </>
  );
}

export default App;