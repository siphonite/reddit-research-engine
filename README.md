Reddit Startup Idea Generator

- Turn any Reddit post into a complete startup idea — including problem, solution, target audience, business model, and potential features — all generated automatically using AI.

Live Demo: [https://redditideas.vercel.app/](https://reddit-ideas-five.vercel.app/)

Backend: Rust (Axum)
Frontend: React + Vite

⭐ Features

- Extracts insights from any Reddit post URL
- Generates a full startup concept
- Clean UI with minimalistic design
- Fast Rust backend deployed on Railway
- React + Vite frontend deployed on Vercel
- Fully responsive UI
- One-click GitHub code access

How It Works:
- Paste a Reddit post link (any subreddit)
- Click Generate
- Get:

- Problem Statement
- Solution
- Business Model
- Key Features
- Differentiation
- Monetization Paths

All generated automatically in seconds.

Tech Stack

Frontend - 
- React (Vite)
- CSS modules
- Fetch API for backend calls

Backend -

- Rust (Axum)
- Reqwest for external calls
- Tokio async runtime
- Railway hosting

Deployment-

- Frontend: Vercel (Free tier)
- Backend: Railway (Free tier)

📁 Project Structure
/frontend
  /src
    /components
    /assets
    App.jsx
    main.jsx
  index.html
  vite.config.js

/backend
  src/main.rs
  Cargo.toml
  Railway.toml

Getting Started (Local Setup)

Clone the repo
git clone https://github.com/Siphonite/reddit_ideas
cd reddit_ideas

Frontend Setup
cd frontend
npm install
npm run dev

Backend Setup
cd backend
cargo run

🌐 Deployment Guide

Frontend (Vercel)

Connect GitHub → Import Repo

Set build command:

npm run build


Set output:

dist

Backend (Railway)

Create project → Deploy from GitHub

Add ENV variables

Railway auto-builds your Rust app

Inspiration

Reddit is full of brilliant problems, pain points, and user frustrations.
This tool turns any such post into a business opportunity with one click.


Future Features (Roadmap)
- Save ideas as PDF
- Add history of generated ideas
- Add more idea analysis (risks, go-to-market, validation steps)
- AI pitch deck generator
- Cleaner UI

Built By

Aman Kumar

Twitter/X: https://x.com/siphonitee

GitHub: https://github.com/Siphonite

⭐ Want to Support?

Give this project a ⭐ on GitHub — it helps a lot!
