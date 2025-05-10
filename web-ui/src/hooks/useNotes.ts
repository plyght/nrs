import { useState, useEffect } from "react";
import { Note } from "../types";

export const useNotes = (searchQuery: string = "") => {
  const [notes, setNotes] = useState<Note[]>([]);
  const [filteredNotes, setFilteredNotes] = useState<Note[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchNotes = async () => {
      try {
        setLoading(true);
        const response = await fetch("/api/notes");

        if (!response.ok) {
          throw new Error(
            `Failed to fetch notes: ${response.status} ${response.statusText}`,
          );
        }

        const data = await response.json();
        setNotes(data);
        setLoading(false);
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "An unknown error occurred",
        );
        setLoading(false);
      }
    };

    fetchNotes();
  }, []);

  // Filter notes when searchQuery or notes change
  useEffect(() => {
    if (!searchQuery.trim()) {
      setFilteredNotes(notes);
      return;
    }

    const query = searchQuery.toLowerCase();
    const filtered = notes.filter(
      (note) =>
        note.title.toLowerCase().includes(query) ||
        note.preview.toLowerCase().includes(query) ||
        note.tags.some((tag) => tag.toLowerCase().includes(query)),
    );

    setFilteredNotes(filtered);
  }, [searchQuery, notes]);

  return { notes: filteredNotes, allNotes: notes, loading, error };
};

export const useNote = (slug: string) => {
  const [note, setNote] = useState<Note | null>(null);
  const [content, setContent] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchNote = async () => {
      try {
        setLoading(true);

        // Fetch note metadata
        const metaResponse = await fetch(`/api/notes/${slug}`);
        if (!metaResponse.ok) {
          throw new Error(
            `Failed to fetch note: ${metaResponse.status} ${metaResponse.statusText}`,
          );
        }
        const noteData = await metaResponse.json();
        setNote(noteData);

        // Fetch note content
        const contentResponse = await fetch(`/notes/${slug}`);
        if (!contentResponse.ok) {
          throw new Error(
            `Failed to fetch note content: ${contentResponse.status} ${contentResponse.statusText}`,
          );
        }
        const content = await contentResponse.text();
        setContent(content);

        setLoading(false);
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "An unknown error occurred",
        );
        setLoading(false);
      }
    };

    if (slug) {
      fetchNote();
    }
  }, [slug]);

  return { note, content, loading, error };
};
