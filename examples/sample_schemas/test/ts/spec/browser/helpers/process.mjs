import process from "process";

process.env = {};

globalThis.process = process;

export default process;
