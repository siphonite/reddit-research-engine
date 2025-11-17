import "./Navbar.css";
import logo from "../assets/logo.png";

function Navbar() {
  return (
    <nav className="navbar">
      {/* Left side: logo + brand text */}
      <div className="nav-left">
        <img src={logo} alt="Logo" className="site-logo" />
        <span className="logo-text">Reddit Ideas</span>
      </div>

      {/* Right side: GitHub button */}
      <div className="nav-right">
        <a
          href="https://github.com/Siphonite/reddit_ideas"
          target="_blank"
          rel="noopener noreferrer"
          className="github-btn"
        >
          GitHub
        </a>
      </div>
    </nav>
  );
}

export default Navbar;
