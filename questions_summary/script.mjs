import fetch from "node-fetch";
import OpenAI from "openai";
import 'dotenv/config'; // This automatically loads .env


const OPENAI_API_KEY = process.env.OPENAI_API_KEY;
const API_BASE = process.env.API_BASE || "https://dev.revillagesociety.org/api";
const AUTH_USER = process.env.API_USER;
const AUTH_PASS = process.env.API_PASS;

const openai = new OpenAI({ apiKey: OPENAI_API_KEY });

async function fetchWeeklyAnswers() {
  const res = await fetch(`${API_BASE}/weekly-answers/all`, {
    headers: {
      "Content-Type": "application/json",
      Authorization:
        AUTH_USER && AUTH_PASS
          ? "Basic " + Buffer.from(`${AUTH_USER}:${AUTH_PASS}`).toString("base64")
          : undefined,
    },
  });

  if (!res.ok) throw new Error(`Failed to fetch answers: ${res.status}`);
  return res.json();
}

async function summarizeAnswers(questionUuid, questionText, answers) {
  const combined = answers.map((a) => `- ${a}`).join("\n");
  const prompt = `
The following are responses to the weekly question:
"${questionText}"

Remove identifying information and profanity.
Summarize the common themes, tones, and insights particularly related to community
 in no more than 150 words.
Use a warm and grounded, human tone.
Responses: `;

  const response = await openai.chat.completions.create({
    model: "gpt-5-nano-2025-08-07",
    messages: [{ role: "user", content: `${prompt} \n${combined}` }],
  });

  const summary = response.choices[0].message.content.trim();
  return { summary, prompt, count: answers.length };
}

async function postSummary(questionUuid, summaryData) {
  const res = await fetch(`${API_BASE}/question-summaries/update`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization:
        AUTH_USER && AUTH_PASS
          ? "Basic " + Buffer.from(`${AUTH_USER}:${AUTH_PASS}`).toString("base64")
          : undefined,
    },
    body: JSON.stringify({
      question_uuid: questionUuid,
      summary: summaryData.summary,
      answers_count: summaryData.count,
      prompt: summaryData.prompt,
    }),
  });

  if (!res.ok) throw new Error(`Failed to post summary: ${res.status}`);
  return res.json();
}

(async function run() {
  try {
    console.log("Fetching grouped weekly answers...");
    const data = await fetchWeeklyAnswers();

    for (const group of data) {
      if (!group.answers || group.answers.length < 3) {
        console.log(`Skipping ${group.question_uuid} (not enough responses)`);
        continue;
      }

      console.log(`Summarizing ${group.question_uuid}: ${group.question}`);
      const summary = await summarizeAnswers(
        group.question_uuid,
        group.question,
        group.answers
      );

      console.log("Posting summary: ", summary.summary);
      await postSummary(group.question_uuid, summary);
    }

    console.log("✅ Summaries completed and uploaded.");
  } catch (err) {
    console.error("❌ Error:", err.message);
  }
})();
