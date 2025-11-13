import { useState } from 'react';
import './App.css';

function App() {
  const [redditUrl, setRedditUrl] = useState('');
  const [result, setResult] = useState(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    setResult(null);
    setLoading(true);

    try {
      const response = await fetch('http://localhost:3000/analyze_post', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ url: redditUrl }),
      });

      if (!response.ok) {
        throw new Error(`Backend error: ${response.statusText}`);
      }

      const data = await response.json();
      setResult(data);
    } catch (err) {
      setError(`Failed to generate ideas: ${err.message}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="App">
      <h1>Reddit Startup Idea Generator</h1>
      <form onSubmit={handleSubmit}>
        <label>
          Reddit Post URL:
          <input
            type="text"
            value={redditUrl}
            onChange={(e) => setRedditUrl(e.target.value)}
            placeholder="https://www.reddit.com/r/..."
            required
          />
        </label>
        <button type="submit" disabled={loading}>
          {loading ? 'Generating...' : 'Generate Ideas'}
        </button>
      </form>
      {error && <p className="error">{error}</p>}
      {result && (
        <div className="result">
          <h2>Post Details</h2>
          <p><strong>Title:</strong> {result.title}</p>
          <p><strong>Body:</strong> {result.body}</p>
          <h2>Startup Ideas</h2>
          <p>{result.ideas}</p>
        </div>
      )}
    </div>
  );
}

export default App;