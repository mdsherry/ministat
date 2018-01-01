options(digits=6)

cat("// Two-sided quantiles: 80%, 90%, 95%, 98%, 99% and 99.5%\n")
cat("pub static T_CONFIDENCES: [&str; 6] = ")
cat("[\"80\", \"90\", \"95\", \"98\", \"99\", \"99.5\"];\n")
cat("#[cfg_attr(rustfmt, rustfmt_skip)]\n")
cat("pub static T_TABLE: [[f64; 6]; 1001] = [\n")
quantiles <- c(0.9, 0.95, 0.975, 0.99, 0.995, 0.9975)
for(dof in 1:1000) {
    critical <- qt(quantiles, df=dof)
    cat("[")
    cat(format(critical, width=6), sep=", ")
    cat("],\n")
}
critical <- qt(quantiles, df=Inf)
    cat("[")
    cat(critical, sep=", ")
    cat("],\n")
cat("];\n")