import { useState, useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { ZoomIn, ZoomOut, RotateCcw, Network } from "lucide-react";
import { motion } from "framer-motion";
import { GraphData, Node, Link } from "../types";
import { Selection, BaseType } from "d3-selection";
import { ZoomBehavior, ZoomTransform } from "d3-zoom";
import { DragBehavior } from "d3-drag";
import { Simulation } from "d3-force";

// Import D3 directly for better TypeScript support
import * as d3 from "d3";

const GraphPage = () => {
  const [graphData, setGraphData] = useState<GraphData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const svgRef = useRef<SVGSVGElement>(null);
  const [width, setWidth] = useState(800);
  const [height, setHeight] = useState(600);
  const navigate = useNavigate();

  // Fetch graph data
  useEffect(() => {
    const fetchGraphData = async () => {
      try {
        setLoading(true);
        const response = await fetch("/api/graph-data");

        if (!response.ok) {
          throw new Error(
            `Failed to fetch graph data: ${response.status} ${response.statusText}`,
          );
        }

        const data = await response.json();
        setGraphData(data);
        setLoading(false);
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "An unknown error occurred",
        );
        setLoading(false);
      }
    };

    fetchGraphData();
  }, []);

  // Update dimensions when window resizes
  useEffect(() => {
    const updateDimensions = () => {
      if (svgRef.current) {
        const container = svgRef.current.parentElement;
        if (container) {
          setWidth(container.clientWidth);
          setHeight(container.clientHeight);
        }
      }
    };

    updateDimensions();
    window.addEventListener("resize", updateDimensions);

    return () => {
      window.removeEventListener("resize", updateDimensions);
    };
  }, []);

  // Initialize and update D3 visualization
  useEffect(() => {
    if (!graphData || !svgRef.current || !d3) return;

    const svg = d3.select<SVGSVGElement, unknown>(svgRef.current);

    // Clear previous graph
    svg.selectAll("*").remove();

    const g = svg.append("g");

    // Create zoom behavior
    const zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.3, 3])
      .on("zoom", (event: any) => {
        g.attr("transform", event.transform);
      });

    // Type assertion to make TypeScript happy
    svg.call(
      zoom as unknown as (
        selection: Selection<SVGSVGElement, unknown, null, undefined>,
      ) => void,
    );

    // Center the graph initially
    svg.call(
      zoom.transform as unknown as (
        selection: Selection<SVGSVGElement, unknown, null, undefined>,
        transform: ZoomTransform,
      ) => void,
      d3.zoomIdentity.translate(width / 2, height / 2).scale(0.8),
    );

    // Define colors
    const isDarkMode = document.documentElement.classList.contains("dark");
    const noteColor = isDarkMode ? "#b497ff" : "#8a70d6";
    const tagColor = isDarkMode ? "#d36eff" : "#d36eff";
    const linkColor = isDarkMode ? "rgba(255,255,255,0.2)" : "rgba(0,0,0,0.2)";
    const textColor = isDarkMode ? "#e0e0e0" : "#333";

    // Create simulation
    const simulation = d3
      .forceSimulation<Node>(graphData.nodes)
      .force(
        "link",
        d3
          .forceLink<Node, Link>(graphData.links)
          .id((d) => d.id)
          .distance(100),
      )
      .force("charge", d3.forceManyBody<Node>().strength(-300))
      .force("center", d3.forceCenter<Node>(0, 0))
      .force(
        "collision",
        d3.forceCollide<Node>().radius((d) => (d.is_tag ? 20 : 15)),
      );

    // Create links
    const link = g
      .selectAll(".link")
      .data(graphData.links)
      .enter()
      .append("path")
      .attr("class", "link")
      .attr("stroke", linkColor)
      .attr("stroke-width", 1.5)
      .attr("fill", "none");

    // Create nodes
    const node = g
      .selectAll(".node")
      .data(graphData.nodes)
      .enter()
      .append("g")
      .attr("class", "node")
      .on("click", (event: any, d: any) => {
        // Navigate to note page when a note node is clicked
        if (!d.is_tag) {
          navigate(`/notes/${d.id}`);
        } else {
          // Navigate to filtered notes for tags
          navigate(`/?search=${encodeURIComponent(d.id.replace(/^#/, ""))}`);
        }
      })
      .call(
        d3
          .drag<SVGGElement, Node>()
          .on("start", dragstarted)
          .on("drag", dragged)
          .on("end", dragended) as unknown as (
          selection: Selection<SVGGElement, Node, SVGGElement, unknown>,
        ) => void,
      );

    // Create node shapes
    node.each(function (this: SVGGElement, d: any) {
      const current = d3.select(this);
      if (d.is_tag) {
        current
          .append("rect")
          .attr("x", -12)
          .attr("y", -8)
          .attr("width", 24)
          .attr("height", 16)
          .attr("rx", 4)
          .attr("fill", tagColor)
          .attr("stroke", "#fff")
          .attr("stroke-width", 1);
      } else {
        current
          .append("circle")
          .attr("r", 8)
          .attr("fill", noteColor)
          .attr("stroke", "#fff")
          .attr("stroke-width", 1);
      }
    });

    // Add text labels
    node
      .append("text")
      .text((d) => d.id)
      .attr("dx", (d) => (d.is_tag ? 15 : 12))
      .attr("dy", 4)
      .attr("font-size", "12px")
      .attr("fill", textColor)
      .style(
        "text-shadow",
        isDarkMode
          ? "1px 1px 2px rgba(0,0,0,0.8)"
          : "1px 1px 2px rgba(255,255,255,0.8)",
      );

    // Update positions on simulation tick
    simulation.on("tick", () => {
      link.attr("d", (d) => {
        // Ensure x and y are defined
        const sourceX = (d.source as Node).x ?? 0;
        const sourceY = (d.source as Node).y ?? 0;
        const targetX = (d.target as Node).x ?? 0;
        const targetY = (d.target as Node).y ?? 0;

        const dx = targetX - sourceX;
        const dy = targetY - sourceY;
        const dr = Math.sqrt(dx * dx + dy * dy) * 1.2;
        return `M${sourceX},${sourceY}A${dr},${dr} 0 0,1 ${targetX},${targetY}`;
      });

      node.attr("transform", (d) => `translate(${d.x ?? 0},${d.y ?? 0})`);
    });

    // Drag functions
    function dragstarted(event: any, d: Node) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      d.fx = d.x;
      d.fy = d.y;
    }

    function dragged(event: any, d: Node) {
      d.fx = event.x;
      d.fy = event.y;
    }

    function dragended(event: any, d: Node) {
      if (!event.active) simulation.alphaTarget(0);
      d.fx = null;
      d.fy = null;
    }

    // Return cleanup function
    return () => {
      simulation.stop();
    };
  }, [graphData, width, height, navigate]);

  // Zoom controls
  const handleZoomIn = () => {
    if (svgRef.current && d3) {
      d3.select<SVGSVGElement, unknown>(svgRef.current)
        .transition()
        .duration(300)
        .call(
          d3.zoom().scaleBy as unknown as (
            transition: d3.Transition<SVGSVGElement, unknown, null, undefined>,
            ...args: any[]
          ) => any,
          1.3,
        );
    }
  };

  const handleZoomOut = () => {
    if (svgRef.current && d3) {
      d3.select<SVGSVGElement, unknown>(svgRef.current)
        .transition()
        .duration(300)
        .call(
          d3.zoom().scaleBy as unknown as (
            transition: d3.Transition<SVGSVGElement, unknown, null, undefined>,
            ...args: any[]
          ) => any,
          0.7,
        );
    }
  };

  const handleReset = () => {
    if (svgRef.current && d3) {
      d3.select<SVGSVGElement, unknown>(svgRef.current)
        .transition()
        .duration(500)
        .call(
          d3.zoom().transform as unknown as (
            transition: d3.Transition<SVGSVGElement, unknown, null, undefined>,
            ...args: any[]
          ) => any,
          d3.zoomIdentity.translate(width / 2, height / 2).scale(0.8),
        );
    }
  };

  if (loading) {
    return (
      <motion.div
        className="flex justify-center items-center min-h-[600px] w-full"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.5 }}
      >
        <div className="flex flex-col items-center">
          <motion.div
            animate={{
              rotateZ: [0, 180, 360],
              scale: [1, 1.1, 1],
            }}
            transition={{
              duration: 2,
              repeat: Infinity,
              ease: "easeInOut",
            }}
          >
            <Network className="h-16 w-16 text-primary-500 dark:text-primary-400 mb-4" />
          </motion.div>
          <motion.div
            initial={{ width: 0 }}
            animate={{ width: 150 }}
            transition={{
              duration: 1,
              repeat: Infinity,
              repeatType: "reverse",
            }}
            className="h-2 bg-gradient-to-r from-primary-300 to-primary-600 rounded-full"
          ></motion.div>
          <p className="mt-4 text-gray-600 dark:text-gray-300">
            Loading graph data...
          </p>
        </div>
      </motion.div>
    );
  }

  if (error || !graphData) {
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
            Failed to Load Graph
          </h2>
          <p className="text-gray-600 dark:text-gray-300 mb-4">
            {error ||
              "Graph data not available. There might be a problem with the server or network connection."}
          </p>
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
      className="flex flex-col min-h-[700px]"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.5 }}
    >
      <motion.div
        className="card flex-grow relative overflow-hidden mb-4 bg-gradient-to-br from-indigo-50 to-purple-50 dark:from-gray-900 dark:to-indigo-950"
        initial={{ y: 20 }}
        animate={{ y: 0 }}
        transition={{ duration: 0.6, delay: 0.1 }}
      >
        <motion.div
          className="absolute top-3 left-3 z-10"
          initial={{ x: -50, opacity: 0 }}
          animate={{ x: 0, opacity: 1 }}
          transition={{ duration: 0.5, delay: 0.3 }}
        >
          <h1 className="text-xl font-bold text-gray-800 dark:text-gray-100 bg-white/70 dark:bg-gray-800/70 backdrop-blur-sm p-2 px-4 rounded-lg shadow-md inline-block">
            <Network className="inline-block mr-2 h-5 w-5" />
            Knowledge Graph
          </h1>
        </motion.div>

        <motion.div
          className="absolute bottom-3 left-3 z-10 flex gap-2"
          initial={{ y: 50, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ duration: 0.5, delay: 0.4 }}
        >
          <motion.button
            onClick={handleZoomIn}
            className="bg-white dark:bg-gray-800 p-2 rounded-full shadow-md hover:bg-indigo-100 dark:hover:bg-indigo-900 transition-colors"
            title="Zoom In"
            whileHover={{ scale: 1.1 }}
            whileTap={{ scale: 0.9 }}
          >
            <ZoomIn
              size={18}
              className="text-indigo-600 dark:text-indigo-400"
            />
          </motion.button>
          <motion.button
            onClick={handleZoomOut}
            className="bg-white dark:bg-gray-800 p-2 rounded-full shadow-md hover:bg-indigo-100 dark:hover:bg-indigo-900 transition-colors"
            title="Zoom Out"
            whileHover={{ scale: 1.1 }}
            whileTap={{ scale: 0.9 }}
          >
            <ZoomOut
              size={18}
              className="text-indigo-600 dark:text-indigo-400"
            />
          </motion.button>
          <motion.button
            onClick={handleReset}
            className="bg-white dark:bg-gray-800 p-2 rounded-full shadow-md hover:bg-indigo-100 dark:hover:bg-indigo-900 transition-colors"
            title="Reset View"
            whileHover={{ scale: 1.1 }}
            whileTap={{ scale: 0.9 }}
          >
            <RotateCcw
              size={18}
              className="text-indigo-600 dark:text-indigo-400"
            />
          </motion.button>
        </motion.div>

        <motion.div
          className="absolute top-3 right-3 z-10 bg-white/70 dark:bg-gray-800/70 backdrop-blur-sm p-2 rounded-lg shadow-md"
          initial={{ x: 50, opacity: 0 }}
          animate={{ x: 0, opacity: 1 }}
          transition={{ duration: 0.5, delay: 0.3 }}
        >
          <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">
            Legend:
          </div>
          <div className="flex flex-col gap-1.5">
            <div className="flex items-center gap-1.5">
              <span className="w-3 h-3 rounded-full bg-indigo-500"></span>
              <span className="text-xs text-gray-700 dark:text-gray-300">
                Notes
              </span>
            </div>
            <div className="flex items-center gap-1.5">
              <span className="w-3 h-3 rounded-sm bg-purple-500 transform rotate-45"></span>
              <span className="text-xs text-gray-700 dark:text-gray-300">
                Tags
              </span>
            </div>
          </div>
        </motion.div>

        <svg
          ref={svgRef}
          width={width}
          height={height}
          className="w-full h-full"
          style={{ cursor: "grab" }}
        />
      </motion.div>

      <motion.div
        className="card p-4 text-sm bg-white dark:bg-gray-800 shadow-lg"
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ duration: 0.5, delay: 0.2 }}
      >
        <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Interact with the graph:
        </h3>
        <div className="flex flex-wrap gap-6">
          <div className="flex items-center gap-1.5">
            <span className="inline-block p-1 bg-blue-100 dark:bg-blue-900 rounded">
              🖱️
            </span>
            <span className="text-gray-600 dark:text-gray-400">
              Drag nodes to rearrange
            </span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="inline-block p-1 bg-blue-100 dark:bg-blue-900 rounded">
              👆
            </span>
            <span className="text-gray-600 dark:text-gray-400">
              Click to navigate to notes
            </span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="inline-block p-1 bg-blue-100 dark:bg-blue-900 rounded">
              🔍
            </span>
            <span className="text-gray-600 dark:text-gray-400">
              Use buttons to zoom and reset
            </span>
          </div>
        </div>
      </motion.div>
    </motion.div>
  );
};

export default GraphPage;
