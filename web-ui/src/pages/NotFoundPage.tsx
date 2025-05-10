import { Link } from "react-router-dom";
import { Home, AlertTriangle } from "lucide-react";
import { motion } from "framer-motion";

const NotFoundPage = () => {
  return (
    <motion.div
      className="flex flex-col items-center justify-center min-h-[60vh] text-center p-4"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.5 }}
    >
      <motion.div
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ type: "spring", stiffness: 300, delay: 0.2 }}
      >
        <motion.div
          animate={{ rotate: [0, 5, 0, -5, 0] }}
          transition={{ repeat: Infinity, duration: 2, repeatDelay: 1 }}
        >
          <AlertTriangle size={80} className="text-yellow-500 mb-8" />
        </motion.div>
      </motion.div>

      <motion.h1
        className="text-4xl font-bold mb-4 text-gray-800 dark:text-gray-100"
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ duration: 0.5, delay: 0.3 }}
      >
        404 - Page Not Found
      </motion.h1>

      <motion.p
        className="text-xl text-gray-600 dark:text-gray-400 mb-8 max-w-md"
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ duration: 0.5, delay: 0.4 }}
      >
        The page you're looking for doesn't exist or has been moved.
      </motion.p>

      <motion.div
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ type: "spring", stiffness: 300, delay: 0.5 }}
      >
        <Link
          to="/"
          className="btn btn-primary flex items-center gap-2 text-lg px-6 py-3 shadow-lg"
        >
          <Home size={20} />
          Back to Home
        </Link>
      </motion.div>
    </motion.div>
  );
};

export default NotFoundPage;
