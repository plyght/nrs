import { useState, useEffect } from "react";
import { useLocation } from "react-router-dom";
import { Search, FileText, Tag, Loader2 } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import NoteCard from "../components/NoteCard";
import { useNotes } from "../hooks/useNotes";

const HomePage = () => {
  const location = useLocation();
  const queryParams = new URLSearchParams(location.search);
  const initialSearchQuery = queryParams.get("search") || "";

  const [searchQuery, setSearchQuery] = useState(initialSearchQuery);
  const [activeTag, setActiveTag] = useState<string | null>(null);
  const [localSearch, setLocalSearch] = useState("");

  const { notes, allNotes, loading, error } = useNotes(searchQuery);

  // Extract all unique tags from notes
  const tags = [...new Set(allNotes.flatMap((note) => note.tags))].sort();

  // Apply tag filter
  const displayedNotes = activeTag
    ? notes.filter((note) => note.tags.includes(activeTag))
    : notes;

  // Update URL when search changes
  useEffect(() => {
    const params = new URLSearchParams(location.search);
    if (searchQuery) {
      params.set("search", searchQuery);
    } else {
      params.delete("search");
    }

    const newSearch = params.toString();
    const newUrl = newSearch
      ? `${location.pathname}?${newSearch}`
      : location.pathname;

    window.history.replaceState(null, "", newUrl);
  }, [searchQuery, location.pathname, location.search]);

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    setSearchQuery(localSearch);
  };

  const handleTagClick = (tag: string) => {
    if (activeTag === tag) {
      setActiveTag(null);
    } else {
      setActiveTag(tag);
    }
  };

  if (loading) {
    return (
      <motion.div
        className="flex justify-center items-center min-h-[400px]"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.5 }}
      >
        <div className="flex flex-col items-center">
          <motion.div
            animate={{ rotate: 360 }}
            transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
          >
            <Loader2 className="h-12 w-12 text-primary-500 dark:text-primary-400" />
          </motion.div>
          <motion.p
            className="mt-4 text-gray-600 dark:text-gray-300"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.3 }}
          >
            Loading your notes...
          </motion.p>
        </div>
      </motion.div>
    );
  }

  if (error) {
    return (
      <motion.div
        className="text-center py-10 max-w-md mx-auto"
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        <div className="card border-red-400 dark:border-red-600 p-6">
          <motion.div
            className="text-red-500 mb-4"
            animate={{ scale: [1, 1.1, 1] }}
            transition={{ duration: 2, repeat: Infinity }}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-16 w-16 mx-auto"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
              />
            </svg>
          </motion.div>
          <h2 className="text-2xl font-bold mb-3 text-gray-800 dark:text-gray-100">
            Failed to Load Notes
          </h2>
          <p className="text-gray-600 dark:text-gray-300 mb-4">{error}</p>
          <motion.button
            className="mt-4 px-4 py-2 bg-primary-500 hover:bg-primary-600 text-white rounded-md shadow-md"
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
            onClick={() => window.location.reload()}
          >
            Try Again
          </motion.button>
        </div>
      </motion.div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.5 }}
    >
      <div className="flex flex-col md:flex-row gap-6">
        {/* Sidebar */}
        <motion.div
          className="md:w-64 flex-shrink-0"
          initial={{ x: -20, opacity: 0 }}
          animate={{ x: 0, opacity: 1 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          {/* Search form */}
          <motion.div
            className="card mb-4 overflow-hidden"
            whileHover={{ boxShadow: "0 8px 20px rgba(0, 0, 0, 0.1)" }}
          >
            <form onSubmit={handleSearch} className="relative">
              <input
                type="text"
                placeholder="Search notes..."
                value={localSearch}
                onChange={(e) => setLocalSearch(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
              />
              <motion.button
                type="submit"
                className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 dark:text-gray-400 hover:text-primary-500 dark:hover:text-primary-400"
                whileHover={{ scale: 1.2 }}
                whileTap={{ scale: 0.9 }}
              >
                <Search size={18} />
              </motion.button>
            </form>
          </motion.div>

          {/* Tags */}
          <motion.div
            className="card bg-white dark:bg-gray-800 shadow-md"
            initial={{ y: 20, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            transition={{ duration: 0.5, delay: 0.2 }}
          >
            <h2 className="text-lg font-semibold text-gray-800 dark:text-gray-100 mb-3 flex items-center gap-2">
              <Tag className="text-primary-500" size={16} />
              Tags
            </h2>

            {tags.length > 0 ? (
              <div className="flex flex-wrap gap-2">
                <AnimatePresence>
                  {tags.map((tag, index) => (
                    <motion.button
                      key={tag}
                      onClick={() => handleTagClick(tag)}
                      className={`tag cursor-pointer transition-colors ${
                        activeTag === tag
                          ? "bg-primary-500 text-white dark:bg-primary-500 dark:text-white"
                          : ""
                      }`}
                      initial={{ opacity: 0, y: 10 }}
                      animate={{ opacity: 1, y: 0 }}
                      transition={{ delay: index * 0.05 }}
                      whileHover={{ scale: 1.05 }}
                      whileTap={{ scale: 0.95 }}
                    >
                      {tag}
                    </motion.button>
                  ))}
                </AnimatePresence>
              </div>
            ) : (
              <motion.p
                className="text-sm text-gray-500 dark:text-gray-400"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
              >
                No tags found
              </motion.p>
            )}
          </motion.div>
        </motion.div>

        {/* Main content */}
        <motion.div
          className="flex-1"
          initial={{ y: 20, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ duration: 0.5, delay: 0.3 }}
        >
          <motion.div
            className="mb-6"
            initial={{ y: -10, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            transition={{ duration: 0.5, delay: 0.4 }}
          >
            <div className="flex items-center justify-between">
              <h1 className="text-2xl font-semibold text-gray-900 dark:text-gray-100 flex items-center gap-2">
                <FileText className="text-primary-500" size={24} />
                Notes
                {activeTag && (
                  <motion.span
                    className="text-lg text-gray-600 dark:text-gray-400"
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.3 }}
                  >
                    / {activeTag}
                  </motion.span>
                )}
              </h1>
              <div className="text-sm text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded-full">
                {displayedNotes.length}{" "}
                {displayedNotes.length === 1 ? "note" : "notes"}
              </div>
            </div>

            {searchQuery && (
              <motion.div
                className="mt-2 p-2 bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded-md text-sm flex justify-between items-center"
                initial={{ height: 0, opacity: 0 }}
                animate={{ height: "auto", opacity: 1 }}
                exit={{ height: 0, opacity: 0 }}
              >
                <span>Search results for: "{searchQuery}"</span>
                <motion.button
                  onClick={() => {
                    setSearchQuery("");
                    setLocalSearch("");
                  }}
                  className="text-blue-600 dark:text-blue-400 hover:underline"
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.95 }}
                >
                  Clear
                </motion.button>
              </motion.div>
            )}
          </motion.div>

          {displayedNotes.length > 0 ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              <AnimatePresence>
                {displayedNotes.map((note, index) => (
                  <motion.div
                    key={note.slug}
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, scale: 0.95 }}
                    transition={{ delay: index * 0.05, duration: 0.3 }}
                  >
                    <NoteCard note={note} />
                  </motion.div>
                ))}
              </AnimatePresence>
            </div>
          ) : (
            <motion.div
              className="card p-8 text-center bg-white dark:bg-gray-800 shadow-md"
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ type: "spring", stiffness: 500, damping: 30 }}
            >
              <motion.div
                className="text-gray-400 dark:text-gray-500 mb-3"
                animate={{ rotateY: [0, 180, 360] }}
                transition={{ duration: 3, repeat: Infinity, repeatDelay: 2 }}
              >
                <FileText size={48} className="mx-auto" />
              </motion.div>
              <h3 className="text-lg font-medium mb-2">No notes found</h3>
              <p className="text-gray-600 dark:text-gray-400 mb-4">
                {searchQuery || activeTag
                  ? "Try adjusting your search or filter criteria"
                  : "You don't have any notes yet"}
              </p>
            </motion.div>
          )}
        </motion.div>
      </div>
    </motion.div>
  );
};

export default HomePage;
