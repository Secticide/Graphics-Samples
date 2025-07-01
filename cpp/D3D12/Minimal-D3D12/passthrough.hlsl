struct vertex_output
{
    float4 position : SV_POSITION;
};

vertex_output vert_main(float3 position : POSITION)
{
    vertex_output output;
    output.position = float4(position, 1.0f);
    return output;
}

float4 frag_main(vertex_output input) : SV_TARGET
{
    return float4(1.0f, 0.0f, 0.0f, 1.0f);
}
